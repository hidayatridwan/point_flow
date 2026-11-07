<script lang="ts">
	import { page } from '$app/state';
	import { resolve } from '$app/paths';
	import * as Collapsible from '$lib/components/ui/collapsible/index.js';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import type { Component } from 'svelte';

	type NavItem = {
		title?: string;
		titleKey?: string;
		url: string;
		icon: Component;
		isActive?: boolean;
		items?: {
			title?: string;
			titleKey?: string;
			url: string;
		}[];
	};

	type NavGroup = {
		title?: string;
		titleKey?: string;
		items: NavItem[];
	};

	let {
		groups,
		badges = {}
	}: {
		groups: NavGroup[];
		badges?: Record<string, number>;
	} = $props();

	function getBadge(url: string) {
		const count = badges[url] ?? 0;
		if (count <= 0) return null;
		return count > 99 ? '99+' : String(count);
	}

	function getItemBadge(item: NavItem) {
		const own = getBadge(item.url);
		if (own) return own;
		const childTotal =
			item.items?.reduce((total, subItem) => total + (badges[subItem.url] ?? 0), 0) ?? 0;
		if (childTotal <= 0) return null;
		return childTotal > 99 ? '99+' : String(childTotal);
	}

	const currentPath = $derived(normalizePath(page.url.pathname));

	function resolveMenuPath(path: string) {
		return resolve(path as '/');
	}

	function isMenuItemActive(item: NavItem) {
		return (
			isPathActive(item.url) || item.items?.some((subItem) => isPathActive(subItem.url)) === true
		);
	}

	function isPathActive(path: string) {
		return currentPath === normalizePath(path);
	}

	function normalizePath(path: string) {
		if (path.length <= 1) return path;
		return path.replace(/\/+$/, '');
	}

	function getTitle(item: { title?: string; titleKey?: string }) {
		return item.titleKey ? t(item.titleKey) : (item.title ?? '');
	}
</script>

{#each groups as group (group.titleKey ?? group.title)}
	<Sidebar.Group>
		<Sidebar.GroupLabel>{getTitle(group)}</Sidebar.GroupLabel>
		<Sidebar.GroupContent>
			<Sidebar.Menu>
				{#each group.items as mainItem (mainItem.titleKey ?? mainItem.title)}
					{@const isActive = mainItem.isActive ?? isMenuItemActive(mainItem)}
					{@const mainItemTitle = getTitle(mainItem)}
					{@const itemBadge = getItemBadge(mainItem)}
					<Collapsible.Root open={isActive}>
						{#snippet child({ props })}
							<Sidebar.MenuItem {...props}>
								<Sidebar.MenuButton tooltipContent={mainItemTitle} {isActive}>
									{#snippet child({ props })}
										<a href={resolveMenuPath(mainItem.url)} {...props}>
											<mainItem.icon />
											<span>{mainItemTitle}</span>
											{#if itemBadge}
												<span
													class="ml-auto inline-flex h-5 min-w-5 items-center justify-center rounded-full bg-primary px-1.5 text-xs font-medium text-primary-foreground tabular-nums"
												>
													{itemBadge}
												</span>
											{/if}
										</a>
									{/snippet}
								</Sidebar.MenuButton>
								{#if mainItem.items?.length}
									<Collapsible.Trigger>
										{#snippet child({ props })}
											<Sidebar.MenuAction {...props} class="data-[state=open]:rotate-90">
												<ChevronRightIcon />
												<span class="sr-only">{t('common.toggle')} {mainItemTitle}</span>
											</Sidebar.MenuAction>
										{/snippet}
									</Collapsible.Trigger>
									<Collapsible.Content>
										<Sidebar.MenuSub>
											{#each mainItem.items as subItem (subItem.titleKey ?? subItem.title)}
												{@const subBadge = getBadge(subItem.url)}
												<Sidebar.MenuSubItem>
													<Sidebar.MenuSubButton
														href={resolveMenuPath(subItem.url)}
														isActive={isPathActive(subItem.url)}
													>
														<span>{getTitle(subItem)}</span>
														{#if subBadge}
															<span
																class="ml-auto inline-flex h-5 min-w-5 items-center justify-center rounded-full bg-primary px-1.5 text-xs font-medium text-primary-foreground tabular-nums"
															>
																{subBadge}
															</span>
														{/if}
													</Sidebar.MenuSubButton>
												</Sidebar.MenuSubItem>
											{/each}
										</Sidebar.MenuSub>
									</Collapsible.Content>
								{/if}
							</Sidebar.MenuItem>
						{/snippet}
					</Collapsible.Root>
				{/each}
			</Sidebar.Menu>
		</Sidebar.GroupContent>
	</Sidebar.Group>
{/each}
