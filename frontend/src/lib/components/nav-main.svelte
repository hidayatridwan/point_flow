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
		groups
	}: {
		groups: NavGroup[];
	} = $props();

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
					<Collapsible.Root open={isActive}>
						{#snippet child({ props })}
							<Sidebar.MenuItem {...props}>
								<Sidebar.MenuButton tooltipContent={mainItemTitle} {isActive}>
									{#snippet child({ props })}
										<a href={resolveMenuPath(mainItem.url)} {...props}>
											<mainItem.icon />
											<span>{mainItemTitle}</span>
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
												<Sidebar.MenuSubItem>
													<Sidebar.MenuSubButton
														href={resolveMenuPath(subItem.url)}
														isActive={isPathActive(subItem.url)}
													>
														<span>{getTitle(subItem)}</span>
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
