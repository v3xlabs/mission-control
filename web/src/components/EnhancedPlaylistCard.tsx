import { FC, useState, useEffect } from "react";
import * as DropdownMenu from "@radix-ui/react-dropdown-menu";
import * as Dialog from "@radix-ui/react-dialog";
import {
  DotsHorizontalIcon,
  PlayIcon,
  PauseIcon,
  Pencil2Icon,
  TrashIcon,
  Cross2Icon,
  PlusIcon
} from "@radix-ui/react-icons";
import { useUpdatePlaylist } from "../hooks/useUpdatePlaylist";
import { useDeletePlaylist } from "../hooks/useDeletePlaylist";
import { useReorderTabs } from "../hooks/useReorderTabs";
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
} from "@dnd-kit/core";
import {
  arrayMove,
  SortableContext,
  sortableKeyboardCoordinates,
  horizontalListSortingStrategy,
} from "@dnd-kit/sortable";
import { restrictToHorizontalAxis } from "@dnd-kit/modifiers";
import { usePlaylistTabs } from "../api/playlists";
import { TabCard } from "./TabCard";
import type { components } from "../api/schema.gen";
import { useStatus } from "../hooks/useStatus";
import { useActivatePlaylist } from "../hooks/useActivatePlaylist";
import { AddTabDialog } from "./AddTabDialog";
import { EditTabDialog } from "./EditTabDialog";

type PlaylistInfo = components["schemas"]["PlaylistInfo"];
type TabInfo = components["schemas"]["TabInfo"];
type UpdatePlaylistRequest = components["schemas"]["UpdatePlaylistRequest"];

interface EnhancedPlaylistCardProps {
  playlist: PlaylistInfo;
}

export const ActivePlaylistIndicator: FC<{ playlistId: string }> = ({ playlistId }) => {
  const { data: status } = useStatus();
  const is_active = status?.current_playlist === playlistId;

  const activatePlaylistMutation = useActivatePlaylist();

  return <button
    onClick={() => activatePlaylistMutation.mutate(playlistId)}
    disabled={activatePlaylistMutation.isPending}
    className={`p-2 rounded-full transition-colors ${is_active
      ? "bg-green-600 text-white"
      : "bg-gray-600 text-gray-300 hover:bg-gray-500"
      } disabled:opacity-50 disabled:cursor-not-allowed`}
  >
    {is_active ? (
      <PauseIcon className="w-4 h-4" />
    ) : (
      <PlayIcon className="w-4 h-4" />
    )}
  </button>
    ;
};

export const EnhancedPlaylistCard: FC<EnhancedPlaylistCardProps> = ({ playlist }) => {
  const [expanded, setExpanded] = useState(true);
  const [editDialogOpen, setEditDialogOpen] = useState(false);
  const [editForm, setEditForm] = useState<UpdatePlaylistRequest>({
    name: playlist.name,
    interval_seconds: playlist.interval_seconds,
  });
  const [editTabDialogOpen, setEditTabDialogOpen] = useState(false);
  const [editingTab, setEditingTab] = useState<TabInfo | null>(null);

  const { data: tabs = [], isLoading: tabsLoading } = usePlaylistTabs(playlist.id);
  const typedTabs = tabs as TabInfo[];

  const updatePlaylistMutation = useUpdatePlaylist({
    onSuccess: () => setEditDialogOpen(false)
  });
  const deletePlaylistMutation = useDeletePlaylist();
  const reorderTabsMutation = useReorderTabs();

  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  );

  const handleDragEnd = (event: any) => {
    const { active, over } = event;

    if (active.id !== over.id) {
      const oldIndex = typedTabs.findIndex((tab) => tab.id === active.id);
      const newIndex = typedTabs.findIndex((tab) => tab.id === over.id);

      const newTabs = arrayMove(typedTabs, oldIndex, newIndex);

      const reorderData = {
        tab_orders: newTabs.map((tab, index) => ({
          tab_id: tab.id,
          order_index: index,
        })),
      };

      reorderTabsMutation.mutate({ playlistId: playlist.id, data: reorderData });
    }
  };

  const handleEditSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    updatePlaylistMutation.mutate({
      playlistId: playlist.id,
      data: {
        name: editForm.name || playlist.name,
        interval_seconds: editForm.interval_seconds || playlist.interval_seconds,
      }
    });
  };

  const handleEditTab = (tab: TabInfo) => {
    setEditingTab(tab);
    setEditTabDialogOpen(true);
  };

  return (
    <div className="bg-gray-800 border border-gray-700 rounded-lg overflow-hidden">
      {/* Playlist Header */}
      <div className="p-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <ActivePlaylistIndicator playlistId={playlist.id} />

            <div>
              <h3 className="text-lg font-semibold text-white">{playlist.name}</h3>
              <p className="text-sm text-gray-400">
                {playlist.tab_count} tabs • {playlist.interval_seconds}s interval
              </p>
            </div>
          </div>

          <div className="flex items-center gap-2">
            {/* Add Tab button when tabs exist */}
            <AddTabDialog
              playlistId={playlist.id}
              tabCount={typedTabs.length}
              trigger={
                <button className="w-full flex items-center justify-center gap-2 px-3 text-sm py-1 bg-gray-700 text-gray-300 rounded-lg hover:bg-gray-600 transition-colors border-2 border-dashed border-gray-600 hover:border-gray-500">
                  <PlusIcon className="w-4 h-4" />
                  Add Tab
                </button>
              }
            />

            <DropdownMenu.Root>
              <DropdownMenu.Trigger asChild>
                <button className="p-1 text-gray-400 hover:text-white transition-colors">
                  <DotsHorizontalIcon className="w-5 h-5" />
                </button>
              </DropdownMenu.Trigger>

              <DropdownMenu.Portal>
                <DropdownMenu.Content
                  className="bg-gray-800 border border-gray-600 rounded-lg shadow-lg p-1 z-50 min-w-[160px]"
                  sideOffset={5}
                >
                  <DropdownMenu.Item
                    onSelect={() => setEditDialogOpen(true)}
                    className="flex items-center gap-2 px-3 py-2 text-sm text-gray-300 hover:bg-gray-700 rounded-md cursor-pointer outline-none"
                  >
                    <Pencil2Icon className="w-4 h-4" />
                    Edit Playlist
                  </DropdownMenu.Item>

                  <DropdownMenu.Separator className="h-px bg-gray-600 m-1" />

                  <DropdownMenu.Item
                    onSelect={() => deletePlaylistMutation.mutate(playlist.id)}
                    disabled={deletePlaylistMutation.isPending}
                    className="flex items-center gap-2 px-3 py-2 text-sm text-red-400 hover:bg-red-900/20 rounded-md cursor-pointer outline-none disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    <TrashIcon className="w-4 h-4" />
                    Delete Playlist
                  </DropdownMenu.Item>
                </DropdownMenu.Content>
              </DropdownMenu.Portal>
            </DropdownMenu.Root>
          </div>
        </div>
      </div>

      {/* Expandable Tab List */}
      {expanded && (
        <div className="border-t border-gray-700 p-4">
          {tabsLoading ? (
            <div className="text-center text-gray-400 py-4">Loading tabs...</div>
          ) : typedTabs.length > 0 ? (
            <>
              <DndContext
                sensors={sensors}
                collisionDetection={closestCenter}
                onDragEnd={handleDragEnd}
                modifiers={[restrictToHorizontalAxis]}
              >
                <SortableContext items={typedTabs.map((t) => t.id)} strategy={horizontalListSortingStrategy}>
                  <div className="flex gap-3 overflow-x-auto pb-2">
                    {typedTabs.map((tab) => (
                      <div key={tab.id} className="flex-shrink-0 w-80">
                        <TabCard
                          tab={tab}
                          playlistId={playlist.id}
                          enabled={true} // TODO: Add enabled field from API
                          onEdit={handleEditTab}
                        />
                      </div>
                    ))}
                  </div>
                </SortableContext>
              </DndContext>


            </>
          ) : (
            <div className="text-center text-gray-400 py-8">
              <p className="mb-2">No tabs in this playlist</p>
              <AddTabDialog playlistId={playlist.id} tabCount={0} />
            </div>
          )}
        </div>
      )}

      {/* Edit Dialog */}
      <Dialog.Root open={editDialogOpen} onOpenChange={setEditDialogOpen}>
        <Dialog.Portal>
          <Dialog.Overlay className="fixed inset-0 bg-black/50 z-50" />
          <Dialog.Content className="fixed top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 bg-gray-800 rounded-lg shadow-xl z-50 w-full max-w-md p-6 border border-gray-700">
            <Dialog.Title className="text-lg font-semibold mb-4 text-gray-100">
              Edit Playlist
            </Dialog.Title>

            <form onSubmit={handleEditSubmit} className="space-y-4">
              <div>
                <label htmlFor="edit-name" className="block text-sm font-medium text-gray-300 mb-1">
                  Display Name
                </label>
                <input
                  id="edit-name"
                  type="text"
                  value={editForm.name || ""}
                  onChange={(e) => setEditForm({ ...editForm, name: e.target.value })}
                  className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-gray-100 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:border-gray-500"
                />
              </div>

              <div>
                <label htmlFor="edit-interval" className="block text-sm font-medium text-gray-300 mb-1">
                  Interval (seconds)
                </label>
                <input
                  id="edit-interval"
                  type="number"
                  min="1"
                  value={editForm.interval_seconds || 30}
                  onChange={(e) => setEditForm({ ...editForm, interval_seconds: parseInt(e.target.value) || 30 })}
                  className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-gray-100 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:border-gray-500"
                />
              </div>

              <div className="flex justify-end gap-3 pt-4">
                <Dialog.Close asChild>
                  <button
                    type="button"
                    className="px-4 py-2 text-gray-300 bg-gray-700 border border-gray-600 rounded-md hover:bg-gray-600 transition-colors"
                  >
                    Cancel
                  </button>
                </Dialog.Close>
                <button
                  type="submit"
                  disabled={updatePlaylistMutation.isPending}
                  className="px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {updatePlaylistMutation.isPending ? "Saving..." : "Save"}
                </button>
              </div>
            </form>

            <Dialog.Close asChild>
              <button
                className="absolute top-4 right-4 text-gray-400 hover:text-gray-600"
                aria-label="Close"
              >
                <Cross2Icon className="w-5 h-5" />
              </button>
            </Dialog.Close>
          </Dialog.Content>
        </Dialog.Portal>
      </Dialog.Root>

      {/* Edit Tab Dialog */}
      <EditTabDialog
        tab={editingTab}
        isOpen={editTabDialogOpen}
        onOpenChange={setEditTabDialogOpen}
      />
    </div>
  );
};