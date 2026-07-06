export { LitElement, html, css, svg, nothing } from 'lit';
export { customElement, property, state } from 'lit/decorators.js';

export type {
  PluginModule,
  PluginContext,
  PluginManifest,
  ViewDescriptor,
  SlotEntry,
  CoreApi,
} from './types';

export { viewRegistry, slotRegistry } from './registry';
export { coreApi } from '../core-api'; // added in Task 7
