<script lang="ts">
  import BookOpen from "@lucide/svelte/icons/book-open";
  import Briefcase from "@lucide/svelte/icons/briefcase";
  import Cpu from "@lucide/svelte/icons/cpu";
  import Crosshair from "@lucide/svelte/icons/crosshair";
  import FileText from "@lucide/svelte/icons/file-text";
  import Moon from "@lucide/svelte/icons/moon";
  import Settings from "@lucide/svelte/icons/settings";
  import Sun from "@lucide/svelte/icons/sun";
  import type { Component } from "svelte";
  import type { NavItem, SectionId } from "../capability";
  import type { Theme } from "../store.svelte";

  interface Props {
    items: NavItem[];
    current: SectionId;
    theme: Theme;
    onSelect: (id: SectionId) => void;
    onTheme: (theme: Theme) => void;
  }

  let { items, current, theme, onSelect, onTheme }: Props = $props();

  const icons: Record<SectionId, Component> = {
    jobs: Briefcase,
    devices: Cpu,
    recipes: BookOpen,
    targets: Crosshair,
    reports: FileText,
    settings: Settings,
  };
</script>

<nav class="rail" aria-label="Sections">
  {#each ["top", "bottom"] as const as group (group)}
    <div class="group">
      {#each items.filter((i) => i.group === group) as item (item.id)}
        {@const Icon = icons[item.id]}
        <button
          class="item"
          class:current={current === item.id}
          aria-label={item.label}
          aria-current={current === item.id ? "page" : undefined}
          aria-describedby={item.reason ? `nav-reason-${item.id}` : undefined}
          title={item.reason ?? item.label}
          disabled={!item.enabled}
          onclick={() => onSelect(item.id)}
        >
          <Icon size={20} strokeWidth={1.5} aria-hidden="true" />
          {#if item.badge}<span class="dot {item.badge}" aria-hidden="true"></span>{/if}
        </button>
        {#if item.reason}<span class="sr-only" id="nav-reason-{item.id}">{item.reason}</span>{/if}
      {/each}
      {#if group === "bottom"}
        <button
          class="item"
          aria-label="Light theme"
          aria-pressed={theme === "light"}
          onclick={() => onTheme("light")}
        >
          <Sun size={20} strokeWidth={1.5} aria-hidden="true" />
        </button>
        <button
          class="item"
          aria-label="Dark theme"
          aria-pressed={theme === "dark"}
          onclick={() => onTheme("dark")}
        >
          <Moon size={20} strokeWidth={1.5} aria-hidden="true" />
        </button>
      {/if}
    </div>
  {/each}
</nav>

<style>
  .rail {
    width: 56px;
    background: var(--surface);
    border-right: 1px solid var(--line);
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    padding: 8px 0;
  }
  .group {
    display: grid;
    gap: 4px;
    justify-items: center;
  }
  .item {
    position: relative;
    width: 40px;
    height: 40px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: var(--muted);
    display: grid;
    place-items: center;
    cursor: pointer;
  }
  .item:hover:not(:disabled) {
    background: var(--subtle);
  }
  .item.current,
  .item[aria-pressed="true"] {
    background: var(--accent-soft);
    color: var(--accent-text);
  }
  .item.current::before {
    content: "";
    position: absolute;
    left: -8px;
    top: 8px;
    bottom: 8px;
    width: 2px;
    background: var(--accent);
  }
  .item:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .dot {
    position: absolute;
    top: 7px;
    right: 7px;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    border: 1.5px solid var(--surface);
  }
  .dot.review { background: var(--review-fg); }
  .dot.warn { background: var(--warn-fg); }
</style>
