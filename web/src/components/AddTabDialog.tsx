import { FC, useState } from "react";
import * as Dialog from "@radix-ui/react-dialog";
import { PlusIcon, Cross2Icon } from "@radix-ui/react-icons";
import { useCreateTab } from "../hooks/useCreateTab";
import { useAddTabToPlaylist } from "../hooks/useAddTabToPlaylist";
import type { components } from "../api/schema.gen";

type CreateTabRequest = components["schemas"]["CreateTabRequest"];

interface AddTabDialogProps {
  playlistId: string;
  tabCount: number;
  trigger?: React.ReactNode;
}

export const AddTabDialog: FC<AddTabDialogProps> = ({ playlistId, tabCount, trigger }) => {
  const [open, setOpen] = useState(false);
  const [formData, setFormData] = useState<CreateTabRequest>({
    id: "",
    name: "",
    url: "",
    persist: true,
  });

  const createTabMutation = useCreateTab({
    onSuccess: (tabId) => {
      // After creating the tab, add it to the playlist
      addToPlaylistMutation.mutate({
        playlistId,
        data: {
          tab_id: tabId,
          order_index: tabCount, // Add at the end
          enabled: true,
        }
      });
    }
  });

  const addToPlaylistMutation = useAddTabToPlaylist({
    onSuccess: () => {
      setOpen(false);
      setFormData({ id: "", name: "", url: "", persist: true });
    }
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (formData.id && formData.name && formData.url) {
      createTabMutation.mutate(formData);
    }
  };

  const isLoading = createTabMutation.isPending || addToPlaylistMutation.isPending;

  return (
    <Dialog.Root open={open} onOpenChange={setOpen}>
      <Dialog.Trigger asChild>
        {trigger || (
          <button className="inline-flex items-center gap-2 px-4 py-2 bg-gray-700 text-white rounded-md hover:bg-gray-600 transition-colors">
            <PlusIcon className="w-4 h-4" />
            Add Tab
          </button>
        )}
      </Dialog.Trigger>

      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/50 z-50" />
        <Dialog.Content className="fixed top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 bg-gray-800 rounded-lg shadow-xl z-50 w-full max-w-md p-6 border border-gray-700">
          <Dialog.Title className="text-lg font-semibold mb-4 text-gray-100">
            Add New Tab
          </Dialog.Title>

          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label htmlFor="tab-id" className="block text-sm font-medium text-gray-300 mb-1">
                Tab ID
              </label>
              <input
                id="tab-id"
                type="text"
                value={formData.id}
                onChange={(e) => setFormData({ ...formData, id: e.target.value })}
                className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-gray-100 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:border-gray-500"
                placeholder="unique-tab-id"
                required
              />
            </div>

            <div>
              <label htmlFor="tab-name" className="block text-sm font-medium text-gray-300 mb-1">
                Display Name
              </label>
              <input
                id="tab-name"
                type="text"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-gray-100 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:border-gray-500"
                placeholder="My Tab"
                required
              />
            </div>

            <div>
              <label htmlFor="tab-url" className="block text-sm font-medium text-gray-300 mb-1">
                URL
              </label>
              <input
                id="tab-url"
                type="url"
                value={formData.url}
                onChange={(e) => setFormData({ ...formData, url: e.target.value })}
                className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-gray-100 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:border-gray-500"
                placeholder="https://example.com"
                required
              />
            </div>

            <div className="flex items-center gap-2">
              <input
                id="tab-persist"
                type="checkbox"
                checked={formData.persist}
                onChange={(e) => setFormData({ ...formData, persist: e.target.checked })}
                className="w-4 h-4 text-green-600 bg-gray-700 border-gray-600 rounded focus:ring-green-500 focus:ring-2"
              />
              <label htmlFor="tab-persist" className="text-sm text-gray-300">
                Persist tab (keep alive when not active)
              </label>
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
                disabled={isLoading}
                className="px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isLoading ? "Adding..." : "Add Tab"}
              </button>
            </div>
          </form>

          <Dialog.Close asChild>
            <button
              className="absolute top-4 right-4 text-gray-400 hover:text-gray-300"
              aria-label="Close"
            >
              <Cross2Icon className="w-5 h-5" />
            </button>
          </Dialog.Close>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
};