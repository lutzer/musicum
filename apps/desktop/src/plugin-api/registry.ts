import type {
  RegisteredSlotEntry,
  RegisteredView,
  SlotEntry,
  ViewDescriptor,
} from './types';

type Listener = () => void;

class Emitter {
  private listeners = new Set<Listener>();
  subscribe(fn: Listener): () => void {
    this.listeners.add(fn);
    return () => this.listeners.delete(fn);
  }
  protected emit(): void {
    for (const fn of this.listeners) fn();
  }
}

export class ViewRegistry extends Emitter {
  private views = new Map<string, RegisteredView>();

  register(bundleId: string, view: ViewDescriptor): void {
    if (this.views.has(view.id)) {
      const existing = this.views.get(view.id)!;
      console.error(
        `[musicum] view id "${view.id}" already registered by "${existing.bundleId}", ` +
        `ignoring duplicate from "${bundleId}"`,
      );
      return;
    }
    this.views.set(view.id, { ...view, bundleId });
    this.emit();
  }

  unregisterBundle(bundleId: string): void {
    let changed = false;
    for (const [id, v] of this.views) {
      if (v.bundleId === bundleId) {
        this.views.delete(id);
        changed = true;
      }
    }
    if (changed) this.emit();
  }

  get(id: string): RegisteredView | undefined {
    return this.views.get(id);
  }

  list(): RegisteredView[] {
    return [...this.views.values()];
  }
}

const DEFAULT_ORDER = 100;

export class SlotRegistry extends Emitter {
  private slots = new Map<string, RegisteredSlotEntry[]>();

  register(bundleId: string, slotId: string, entry: SlotEntry): void {
    const list = this.slots.get(slotId) ?? [];
    if (list.some(e => e.id === entry.id)) {
      const existing = list.find(e => e.id === entry.id)!;
      console.error(
        `[musicum] slot entry "${slotId}/${entry.id}" already registered by ` +
        `"${existing.bundleId}", ignoring duplicate from "${bundleId}"`,
      );
      return;
    }
    list.push({
      ...entry,
      order: entry.order ?? DEFAULT_ORDER,
      bundleId,
      slotId,
    });
    list.sort((a, b) => (a.order ?? DEFAULT_ORDER) - (b.order ?? DEFAULT_ORDER));
    this.slots.set(slotId, list);
    this.emit();
  }

  unregisterBundle(bundleId: string): void {
    let changed = false;
    for (const [slotId, list] of this.slots) {
      const next = list.filter(e => e.bundleId !== bundleId);
      if (next.length !== list.length) {
        this.slots.set(slotId, next);
        changed = true;
      }
    }
    if (changed) this.emit();
  }

  entries(slotId: string): readonly RegisteredSlotEntry[] {
    return this.slots.get(slotId) ?? [];
  }
}

// Singletons used by the shell and the plugin loader.
export const viewRegistry = new ViewRegistry();
export const slotRegistry = new SlotRegistry();
