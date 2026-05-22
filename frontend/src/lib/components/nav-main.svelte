<script lang="ts">
	import { page } from '$app/state';
	import { resolve } from '$app/paths';
	import type { PathnameWithSearchOrHash } from '$app/types';
	import * as Collapsible from '$lib/components/ui/collapsible/index.js';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import type { Component } from 'svelte';

	type NavItem = {
		title: string;
		url: string;
		icon: Component;
		isActive?: boolean;
		items?: {
			title: string;
			url: string;
		}[];
	};

	type NavGroup = {
		title: string;
		items: NavItem[];
	};

	let {
		groups
	}: {
		groups: NavGroup[];
	} = $props();

	const currentPath = $derived(normalizePath(page.url.pathname));

	function resolveMenuPath(path: string) {
		return resolve(path as PathnameWithSearchOrHash);
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
</script>

{#each groups as group (group.title)}
	<Sidebar.Group>
		<Sidebar.GroupLabel>{group.title}</Sidebar.GroupLabel>
		<Sidebar.GroupContent>
			<Sidebar.Menu>
				{#each group.items as mainItem (mainItem.title)}
					{@const isActive = mainItem.isActive ?? isMenuItemActive(mainItem)}
					<Collapsible.Root open={isActive}>
						{#snippet child({ props })}
							<Sidebar.MenuItem {...props}>
								<Sidebar.MenuButton tooltipContent={mainItem.title} {isActive}>
									{#snippet child({ props })}
										<a href={resolveMenuPath(mainItem.url)} {...props}>
											<mainItem.icon />
											<span>{mainItem.title}</span>
										</a>
									{/snippet}
								</Sidebar.MenuButton>
								{#if mainItem.items?.length}
									<Collapsible.Trigger>
										{#snippet child({ props })}
											<Sidebar.MenuAction {...props} class="data-[state=open]:rotate-90">
												<ChevronRightIcon />
												<span class="sr-only">Toggle {mainItem.title}</span>
											</Sidebar.MenuAction>
										{/snippet}
									</Collapsible.Trigger>
									<Collapsible.Content>
										<Sidebar.MenuSub>
											{#each mainItem.items as subItem (subItem.title)}
												<Sidebar.MenuSubItem>
													<Sidebar.MenuSubButton
														href={resolveMenuPath(subItem.url)}
														isActive={isPathActive(subItem.url)}
													>
														<span>{subItem.title}</span>
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
