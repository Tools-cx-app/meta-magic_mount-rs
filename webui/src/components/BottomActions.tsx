import type { ParentProps } from "solid-js";

export default (props: ParentProps) => (
  <div class="bottom-actions-root">
    {props.children}
    <style>
      {`
      .bottom-actions-root {
        position: sticky;
        bottom: 0;
        left: 0;
        right: 0;
        display: flex;
        align-items: center;
        padding: 0px;
        gap: 16px;
        z-index: 90;
        pointer-events: none;
        margin-top: auto;
      }
      .bottom-actions-root > * {
        pointer-events: auto;
      }
      .bottom-actions-root > .spacer {
        flex: 1;
        pointer-events: none;
        box-shadow: none;
      }
      `}
    </style>
  </div>
);
