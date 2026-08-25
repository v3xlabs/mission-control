import { closestCenter, DndContext, type DragEndEvent } from "@dnd-kit/core";
import { horizontalListSortingStrategy, SortableContext } from "@dnd-kit/sortable";
import { type FC } from "react";
import { FiTrash2 } from "react-icons/fi";

import { usePlaylistTabs } from "../api/playlists";
import type { components } from "../api/schema.gen";
import { useActivatePlaylist } from "../hooks/useActivatePlaylist";
import { useDeletePlaylist } from "../hooks/useDeletePlaylist";
import { useReorderTabs } from "../hooks/useReorderTabs";
import { useStatus } from "../hooks/useStatus";
import { AddTabDialog } from "./AddTabDialog";
import { TabCard } from "./TabCard";

type PlaylistInfo = components["schemas"]["PlaylistInfo"];

export const PlaylistCard: FC<{ playlist: PlaylistInfo; }> = ({ playlist }) => {
  const { data: status } = useStatus();
  const { data: tabs = [], isLoading } = usePlaylistTabs(playlist.playlist_id);

  const reorder = useReorderTabs();
  const activate = useActivatePlaylist();
  const remove = useDeletePlaylist();

  const isActive = status?.current_playlist_id === playlist.playlist_id;

  const onDragEnd = ({ active, over }: DragEndEvent) => {
    // dnd-kit identifies a sortable by `id`. The name belongs to its API, not to this codebase.
    /* eslint-disable no-restricted-syntax */
    if (!over || active.id === over.id) {
      return;
    }

    const from = tabs.findIndex(tab => tab.tab_id === active.id);
    const to = tabs.findIndex(tab => tab.tab_id === over.id);
    /* eslint-enable no-restricted-syntax */

    if (from === -1 || to === -1) {
      return;
    }

    const tabIds = tabs.map(tab => tab.tab_id);
    const [moved] = tabIds.splice(from, 1);

    tabIds.splice(to, 0, moved);

    reorder.mutate({ playlistId: playlist.playlist_id, tabIds });
  };

  return (
    <section className="border border-gray-800">
      <header className="flex items-center gap-3 border-b border-gray-800 px-4 py-3">
        <h2 className="font-semibold text-gray-100">{playlist.name}</h2>
        {isActive && (
          <span className="bg-emerald-600 px-1.5 py-0.5 text-xs text-white">active</span>
        )}
        {playlist.is_default && (
          <span className="bg-gray-700 px-1.5 py-0.5 text-xs text-gray-200">default</span>
        )}
        <span className="text-xs text-gray-500">
          {`every ${playlist.interval}, ${playlist.tab_count} tabs`}
        </span>
        <span className="flex-1" />
        <AddTabDialog playlistId={playlist.playlist_id} />
        {!isActive && (
          <button
            type="button"
            onClick={() => activate.mutate(playlist.playlist_id)}
            className="bg-gray-800 px-2 py-1 text-sm text-gray-100 hover:bg-gray-700"
          >
            Play
          </button>
        )}
        <button
          type="button"
          onClick={() => remove.mutate(playlist.playlist_id)}
          className="text-gray-500 hover:text-red-400"
          title="Delete playlist"
        >
          <FiTrash2 />
        </button>
      </header>

      <div className="overflow-x-auto p-4">
        {isLoading && <p className="text-sm text-gray-500">Loading tabs...</p>}
        {!isLoading && tabs.length === 0 && (
          <p className="text-sm text-gray-500">No tabs yet.</p>
        )}
        <DndContext collisionDetection={closestCenter} onDragEnd={onDragEnd}>
          <SortableContext
            items={tabs.map(tab => tab.tab_id)}
            strategy={horizontalListSortingStrategy}
          >
            <div className="flex gap-3">
              {tabs.map(tab => (
                <TabCard
                  key={tab.tab_id}
                  tab={tab}
                  playlistId={playlist.playlist_id}
                  isOnScreen={isActive && status?.current_tab_id === tab.tab_id}
                />
              ))}
            </div>
          </SortableContext>
        </DndContext>
      </div>
    </section>
  );
};
