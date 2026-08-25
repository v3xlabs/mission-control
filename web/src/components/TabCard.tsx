import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import * as Switch from "@radix-ui/react-switch";
import { type FC } from "react";
import { FiExternalLink, FiRefreshCw, FiRotateCw, FiTrash2 } from "react-icons/fi";

import type { components } from "../api/schema.gen";
import { useActivateTab } from "../hooks/useActivateTab";
import { useRecreateTab } from "../hooks/useRecreateTab";
import { useRefreshTab } from "../hooks/useRefreshTab";
import { useRemoveTabFromPlaylist } from "../hooks/useRemoveTabFromPlaylist";
import { useSetTabEnabled } from "../hooks/useSetTabEnabled";
import { TabPreview } from "./TabPreview";

type TabInfo = components["schemas"]["TabInfo"];

type Properties = {
  tab: TabInfo;
  playlistId: string;
  isOnScreen: boolean;
};

export const TabCard: FC<Properties> = ({ tab, playlistId, isOnScreen }) => {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    // eslint-disable-next-line no-restricted-syntax -- dnd-kit names this field.
    id: tab.tab_id,
  });

  const activate = useActivateTab();
  const refresh = useRefreshTab();
  const recreate = useRecreateTab();
  const setEnabled = useSetTabEnabled();
  const remove = useRemoveTabFromPlaylist();

  return (
    <article
      ref={setNodeRef}
      style={{ transform: CSS.Transform.toString(transform), transition }}
      className={[
        "flex w-72 shrink-0 flex-col gap-2 border p-3",
        isOnScreen ? "border-emerald-500" : "border-gray-800",
        isDragging ? "opacity-50" : "",
        tab.enabled ? "" : "opacity-60",
      ].join(" ")}
    >
      <div className="flex items-start gap-2">
        <button
          type="button"
          className="cursor-grab text-gray-500"
          aria-label="Reorder tab"
          {...attributes}
          {...listeners}
        >
          ⠿
        </button>
        <div className="min-w-0 flex-1">
          <h3 className="truncate font-medium text-gray-100">{tab.name}</h3>
          <p className="truncate text-xs text-gray-500">{tab.url}</p>
        </div>
        <Switch.Root
          checked={tab.enabled}
          onCheckedChange={enabled => setEnabled.mutate({
            playlistId,
            tabId: tab.tab_id,
            enabled,
          })}
          className="h-5 w-9 shrink-0 bg-gray-700 data-[state=checked]:bg-emerald-600"
          aria-label={tab.enabled ? "Disable tab" : "Enable tab"}
        >
          <Switch.Thumb className="block h-4 w-4 translate-x-0.5 bg-white transition-transform data-[state=checked]:translate-x-4" />
        </Switch.Root>
      </div>

      <button
        type="button"
        onClick={() => activate.mutate({ playlistId, tabId: tab.tab_id })}
        className="relative block aspect-video w-full overflow-hidden bg-gray-900"
        title="Put this tab on screen"
      >
        <TabPreview tabId={tab.tab_id} />
        {isOnScreen && (
          <span className="absolute top-1 left-1 bg-emerald-600 px-1.5 py-0.5 text-xs text-white">
            on screen
          </span>
        )}
      </button>

      <div className="flex items-center gap-3 text-gray-400">
        <button type="button" onClick={() => refresh.mutate(tab.tab_id)} title="Reload the page">
          <FiRefreshCw />
        </button>
        <button type="button" onClick={() => recreate.mutate(tab.tab_id)} title="Close and reopen the page">
          <FiRotateCw />
        </button>
        <a href={tab.url} target="_blank" rel="noreferrer" title="Open in this browser">
          <FiExternalLink />
        </a>
        <span className="flex-1" />
        <button
          type="button"
          onClick={() => remove.mutate({ playlistId, tabId: tab.tab_id })}
          title="Remove from this playlist"
          className="hover:text-red-400"
        >
          <FiTrash2 />
        </button>
      </div>
    </article>
  );
};
