use serde_json::json;
use sqlx::PgPool;

use crate::{
    auth::AuthUser,
    error::{AppError, AppResult},
    models::{
        BucketAllocation, CancelRequest, CreateProfitSharingPeriodRequest, DistributionQuery,
        ListQuery, Paginated, ProfitSharingDistributionResponse, ProfitSharingPeriodDetailResponse,
        ProfitSharingPeriodResponse, UpdateProfitSharingPeriodRequest,
    },
    services::{reports, validate_required},
};

use super::repository::{self, MemberNetRow};

/// Input net per anggota untuk kalkulasi murni (tanpa database).
#[derive(Debug, Clone)]
pub struct MemberNetInput {
    pub member_id: i64,
    pub is_active: bool,
    pub gross_sales: f64,
    pub returns_amount: f64,
}

/// Hasil kalkulasi per anggota, siap di-snapshot ke distribusi.
#[derive(Debug, Clone, PartialEq)]
pub struct MemberShare {
    pub member_id: i64,
    pub transaction_amount: f64,
    pub share_pct: f64,
    pub share_amount: f64,
}

/// Kalkulasi distribusi murni (desain Bagian 4.2): `member_pool` dibagi
/// proporsional net (penjualan − retur) anggota aktif bernet positif.
/// Murni agar bisa diuji unit.
pub fn calculate_member_shares(
    pool: f64,
    member_pct: f64,
    inputs: &[MemberNetInput],
) -> Vec<MemberShare> {
    let member_pool = pool * member_pct / 100.0;
    let nets: Vec<(i64, f64)> = inputs
        .iter()
        .filter(|input| input.is_active)
        .map(|input| (input.member_id, input.gross_sales - input.returns_amount))
        .filter(|(_, net)| *net > 0.0)
        .collect();
    let total: f64 = nets.iter().map(|(_, net)| net).sum();
    if total <= 0.0 {
        return Vec::new();
    }

    nets.into_iter()
        .map(|(member_id, net)| {
            let share_pct = net / total;
            MemberShare {
                member_id,
                transaction_amount: net,
                share_pct,
                share_amount: member_pool * share_pct,
            }
        })
        .collect()
}

fn validate_period_name(name: &str) -> AppResult<String> {
    validate_required(name, "period_name")?;
    Ok(name.trim().to_string())
}

fn validate_dates(
    start: chrono::NaiveDate,
    end: chrono::NaiveDate,
) -> AppResult<(chrono::NaiveDate, chrono::NaiveDate)> {
    if start > end {
        return Err(AppError::Validation(
            "start_date must not be after end_date".to_string(),
        ));
    }
    Ok((start, end))
}

fn validate_percentage(value: f64, field: &str) -> AppResult<f64> {
    if !value.is_finite() || value < 0.0 || value > 100.0 {
        return Err(AppError::Validation(format!(
            "{field} must be between 0 and 100"
        )));
    }
    Ok((value * 100.0).round() / 100.0)
}

fn validate_percentage_sum(pcts: &[f64; 6]) -> AppResult<()> {
    let sum: f64 = pcts.iter().sum();
    if (sum - 100.0).abs() > 1e-6 {
        return Err(AppError::Validation(
            "allocation percentages must total 100".to_string(),
        ));
    }
    Ok(())
}

fn validate_pool(pool: Option<f64>) -> AppResult<Option<f64>> {
    match pool {
        Some(value) => {
            if !value.is_finite() || value < 0.0 {
                return Err(AppError::Validation(
                    "total_pool must be zero or greater".to_string(),
                ));
            }
            Ok(Some((value * 100.0).round() / 100.0))
        }
        None => Ok(None),
    }
}

fn period_pcts(period: &ProfitSharingPeriodResponse) -> [f64; 6] {
    [
        period.member_pct,
        period.reserve_pct,
        period.management_pct,
        period.supervisor_pct,
        period.social_pct,
        period.education_pct,
    ]
}

pub async fn list_periods(
    pool: &PgPool,
    query: ListQuery,
) -> AppResult<Paginated<ProfitSharingPeriodResponse>> {
    let (data, meta) = repository::list_periods(pool, &query).await?;
    Ok(Paginated { data, meta })
}

pub async fn find_period_detail(
    pool: &PgPool,
    id: i64,
) -> AppResult<ProfitSharingPeriodDetailResponse> {
    let period = repository::find_period(pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    build_detail(pool, period).await
}

async fn build_detail(
    pool: &PgPool,
    period: ProfitSharingPeriodResponse,
) -> AppResult<ProfitSharingPeriodDetailResponse> {
    let pool_used = period.total_pool.or(period.auto_net_profit).unwrap_or(0.0);
    let member_pool = pool_used * period.member_pct / 100.0;
    let (member_count, total_member_transaction) =
        repository::distribution_summary(pool, period.id).await?;
    let allocations = vec![
        BucketAllocation {
            key: "member",
            percentage: period.member_pct,
            amount: member_pool,
        },
        BucketAllocation {
            key: "reserve",
            percentage: period.reserve_pct,
            amount: pool_used * period.reserve_pct / 100.0,
        },
        BucketAllocation {
            key: "management",
            percentage: period.management_pct,
            amount: pool_used * period.management_pct / 100.0,
        },
        BucketAllocation {
            key: "supervisor",
            percentage: period.supervisor_pct,
            amount: pool_used * period.supervisor_pct / 100.0,
        },
        BucketAllocation {
            key: "social",
            percentage: period.social_pct,
            amount: pool_used * period.social_pct / 100.0,
        },
        BucketAllocation {
            key: "education",
            percentage: period.education_pct,
            amount: pool_used * period.education_pct / 100.0,
        },
    ];

    Ok(ProfitSharingPeriodDetailResponse {
        period,
        pool_used,
        member_pool,
        member_count,
        total_member_transaction,
        allocations,
    })
}

pub async fn create_period(
    pool: &PgPool,
    user: &AuthUser,
    request: CreateProfitSharingPeriodRequest,
) -> AppResult<ProfitSharingPeriodDetailResponse> {
    let period_name = validate_period_name(&request.period_name)?;
    let (start_date, end_date) = validate_dates(request.start_date, request.end_date)?;
    let pcts = [
        validate_percentage(request.member_pct, "member_pct")?,
        validate_percentage(request.reserve_pct, "reserve_pct")?,
        validate_percentage(request.management_pct, "management_pct")?,
        validate_percentage(request.supervisor_pct, "supervisor_pct")?,
        validate_percentage(request.social_pct, "social_pct")?,
        validate_percentage(request.education_pct, "education_pct")?,
    ];
    validate_percentage_sum(&pcts)?;
    let total_pool = validate_pool(request.total_pool)?;

    let period = repository::create_period(
        pool,
        user,
        CreateProfitSharingPeriodRequest {
            period_name,
            start_date,
            end_date,
            total_pool,
            member_pct: pcts[0],
            reserve_pct: pcts[1],
            management_pct: pcts[2],
            supervisor_pct: pcts[3],
            social_pct: pcts[4],
            education_pct: pcts[5],
            notes: request.notes,
        },
    )
    .await?;

    build_detail(pool, period).await
}

pub async fn update_period(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
    request: UpdateProfitSharingPeriodRequest,
) -> AppResult<ProfitSharingPeriodDetailResponse> {
    let current = repository::find_period(pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    if current.status != "draft" {
        return Err(AppError::Conflict(
            "only draft periods can be edited".to_string(),
        ));
    }

    let period_name = request
        .period_name
        .map(|value| validate_period_name(&value))
        .transpose()?;
    let start_date = request.start_date.unwrap_or(current.start_date);
    let end_date = request.end_date.unwrap_or(current.end_date);
    let (start_date, end_date) = validate_dates(start_date, end_date)?;

    let current_pcts = period_pcts(&current);
    let patched = [
        request.member_pct,
        request.reserve_pct,
        request.management_pct,
        request.supervisor_pct,
        request.social_pct,
        request.education_pct,
    ];
    let mut pcts = current_pcts;
    for (index, value) in patched.into_iter().enumerate() {
        if let Some(value) = value {
            pcts[index] = validate_percentage(value, "percentage")?;
        }
    }
    validate_percentage_sum(&pcts)?;

    let (clear_pool, total_pool) = match request.total_pool {
        None => (false, None),
        Some(value) => (value.is_none(), validate_pool(value)?),
    };

    let period = repository::update_period(
        pool,
        user,
        id,
        period_name,
        Some(start_date),
        Some(end_date),
        clear_pool,
        total_pool,
        Some(pcts[0]),
        Some(pcts[1]),
        Some(pcts[2]),
        Some(pcts[3]),
        Some(pcts[4]),
        Some(pcts[5]),
        request.notes,
    )
    .await?;

    build_detail(pool, period).await
}

pub async fn calculate_period(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
) -> AppResult<ProfitSharingPeriodDetailResponse> {
    let current = repository::find_period(pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    if current.status != "draft" && current.status != "calculated" {
        return Err(AppError::Conflict(
            "only draft or calculated periods can be calculated".to_string(),
        ));
    }

    // Laba bersih periode via income-statement (desain Bagian 4.2).
    let income =
        reports::income_statement(pool, Some(current.start_date), Some(current.end_date)).await?;
    let pool_used = current.total_pool.unwrap_or(income.net_profit);

    let nets = repository::member_nets(pool, current.start_date, current.end_date).await?;
    let inputs: Vec<MemberNetInput> = nets
        .into_iter()
        .map(|row: MemberNetRow| MemberNetInput {
            member_id: row.member_id,
            is_active: true,
            gross_sales: row.gross_sales,
            returns_amount: row.returns_amount,
        })
        .collect();
    let shares = calculate_member_shares(pool_used, current.member_pct, &inputs);

    let mut tx = pool.begin().await?;
    let locked = repository::lock_period(&mut tx, id)
        .await?
        .ok_or(AppError::NotFound)?;
    if locked.status != "draft" && locked.status != "calculated" {
        return Err(AppError::Conflict(
            "only draft or calculated periods can be calculated".to_string(),
        ));
    }
    let from_draft = locked.status == "draft";
    repository::replace_distributions(
        &mut tx,
        id,
        &shares
            .iter()
            .map(|share| repository::NewDistribution {
                member_id: share.member_id,
                transaction_amount: share.transaction_amount,
                share_pct: share.share_pct,
                share_amount: share.share_amount,
            })
            .collect::<Vec<_>>(),
    )
    .await?;
    repository::mark_calculated(&mut tx, id, income.net_profit, from_draft).await?;
    tx.commit().await?;

    repository::record_action_audit(
        pool,
        user,
        "profit_sharing.calculate",
        id,
        json!({ "pool_used": pool_used, "member_count": shares.len() }),
    )
    .await;

    find_period_detail(pool, id).await
}

pub async fn finalize_period(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
) -> AppResult<ProfitSharingPeriodDetailResponse> {
    let current = repository::find_period(pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    if current.status != "calculated" {
        return Err(AppError::Conflict(
            "only calculated periods can be finalized".to_string(),
        ));
    }

    let mut tx = pool.begin().await?;
    let locked = repository::lock_period(&mut tx, id)
        .await?
        .ok_or(AppError::NotFound)?;
    if locked.status != "calculated" {
        return Err(AppError::Conflict(
            "only calculated periods can be finalized".to_string(),
        ));
    }
    repository::mark_finalized(&mut tx, id, user.id).await?;
    tx.commit().await?;

    repository::record_action_audit(pool, user, "profit_sharing.finalize", id, json!({})).await;

    find_period_detail(pool, id).await
}

pub async fn cancel_period(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
    request: CancelRequest,
) -> AppResult<ProfitSharingPeriodDetailResponse> {
    let reason = request.reason.unwrap_or_default().trim().to_string();
    validate_required(&reason, "reason")?;

    let current = repository::find_period(pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    if current.status != "draft" && current.status != "calculated" {
        return Err(AppError::Conflict(
            "only draft or calculated periods can be cancelled".to_string(),
        ));
    }

    let mut tx = pool.begin().await?;
    let locked = repository::lock_period(&mut tx, id)
        .await?
        .ok_or(AppError::NotFound)?;
    if locked.status != "draft" && locked.status != "calculated" {
        return Err(AppError::Conflict(
            "only draft or calculated periods can be cancelled".to_string(),
        ));
    }
    repository::mark_cancelled(&mut tx, id, user.id, &reason).await?;
    tx.commit().await?;

    repository::record_action_audit(
        pool,
        user,
        "profit_sharing.cancel",
        id,
        json!({ "reason": reason }),
    )
    .await;

    find_period_detail(pool, id).await
}

pub async fn list_distributions(
    pool: &PgPool,
    id: i64,
    query: DistributionQuery,
) -> AppResult<Paginated<ProfitSharingDistributionResponse>> {
    repository::find_period(pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    let (data, meta) = repository::list_distributions(pool, id, &query).await?;
    Ok(Paginated { data, meta })
}

pub async fn export_distributions_csv(pool: &PgPool, id: i64) -> AppResult<String> {
    repository::find_period(pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    let rows = repository::export_distributions(pool, id).await?;

    let mut csv =
        String::from("member_no,full_name,phone,transaction_amount,share_pct,share_amount\n");
    for row in rows {
        csv.push_str(&format!(
            "{},{},{},{:.2},{:.4},{:.2}\n",
            csv_cell(&row.member_no),
            csv_cell(&row.full_name),
            csv_cell(&row.phone),
            row.transaction_amount,
            row.share_pct,
            row.share_amount
        ));
    }

    Ok(csv)
}

fn csv_cell(value: &str) -> String {
    if value.contains([',', '"', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(member_id: i64, gross: f64, returns: f64) -> MemberNetInput {
        MemberNetInput {
            member_id,
            is_active: true,
            gross_sales: gross,
            returns_amount: returns,
        }
    }

    #[test]
    fn manual_pool_scales_shares_proportionally() {
        let shares = calculate_member_shares(
            1_000_000.0,
            50.0,
            &[input(1, 300_000.0, 0.0), input(2, 100_000.0, 0.0)],
        );

        assert_eq!(shares.len(), 2);
        assert!((shares[0].share_pct - 0.75).abs() < 1e-9);
        assert!((shares[0].share_amount - 375_000.0).abs() < 1e-6);
        assert!((shares[1].share_amount - 125_000.0).abs() < 1e-6);
        let total: f64 = shares.iter().map(|share| share.share_amount).sum();
        assert!((total - 500_000.0).abs() < 1e-6);
    }

    #[test]
    fn auto_pool_value_flows_into_member_pool() {
        // Pool auto (laba bersih) vs manual: hanya nilai pool yang berbeda.
        let auto = calculate_member_shares(365_000.0, 50.0, &[input(1, 200_000.0, 0.0)]);
        let manual = calculate_member_shares(1_000_000.0, 50.0, &[input(1, 200_000.0, 0.0)]);

        assert_eq!(auto.len(), 1);
        assert_eq!(auto[0].share_pct, 1.0);
        assert!((auto[0].share_amount - 182_500.0).abs() < 1e-6);
        assert!((manual[0].share_amount - 500_000.0).abs() < 1e-6);
    }

    #[test]
    fn empty_nets_yield_no_distributions() {
        assert!(calculate_member_shares(1_000_000.0, 50.0, &[]).is_empty());
        assert!(calculate_member_shares(1_000_000.0, 50.0, &[input(1, 0.0, 0.0)]).is_empty());
    }

    #[test]
    fn inactive_members_are_excluded() {
        let inactive = MemberNetInput {
            member_id: 9,
            is_active: false,
            gross_sales: 999_999.0,
            returns_amount: 0.0,
        };

        let shares = calculate_member_shares(1_000_000.0, 50.0, &[inactive, input(1, 100.0, 0.0)]);

        assert_eq!(shares.len(), 1);
        assert_eq!(shares[0].member_id, 1);
        assert_eq!(shares[0].share_pct, 1.0);
    }

    #[test]
    fn returns_reduce_contribution() {
        // Net sama (150rb) walau komposisi beda → bagian sama.
        let shares = calculate_member_shares(
            1_000_000.0,
            50.0,
            &[input(1, 200_000.0, 50_000.0), input(2, 150_000.0, 0.0)],
        );

        assert_eq!(shares.len(), 2);
        assert!((shares[0].transaction_amount - 150_000.0).abs() < 1e-6);
        assert!((shares[0].share_amount - shares[1].share_amount).abs() < 1e-6);
    }

    #[test]
    fn non_positive_nets_are_excluded() {
        // Retur melebihi penjualan → net negatif → tidak masuk distribusi.
        let shares = calculate_member_shares(
            1_000_000.0,
            50.0,
            &[input(1, 10_000.0, 50_000.0), input(2, 100_000.0, 0.0)],
        );

        assert_eq!(shares.len(), 1);
        assert_eq!(shares[0].member_id, 2);
    }
}
