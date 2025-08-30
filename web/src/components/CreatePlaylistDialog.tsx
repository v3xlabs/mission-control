import { FC, useState } from "react";
import * as Dialog from "@radix-ui/react-dialog";
import { PlusIcon, Cross2Icon } from "@radix-ui/react-icons";
import { useCreatePlaylist } from "../hooks/useCreatePlaylist";
import type { components } from "../api/schema.gen";

type CreatePlaylistRequest = components["schemas"]["CreatePlaylistRequest"];

interface CreatePlaylistDialogProps {
  trigger?: React.ReactNode;
}

export const CreatePlaylistDialog: FC<CreatePlaylistDialogProps> = ({ trigger }) => {
  const [open, setOpen] = useState(false);
  const [formData, setFormData] = useState<CreatePlaylistRequest>({
    id: "",
    name: "",
    interval_seconds: 30,
  });

  const createPlaylistMutation = useCreatePlaylist({
    onSuccess: () => {
      setOpen(false);
      setFormData({ id: "", name: "", interval_seconds: 30 });
    }
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (formData.id && formData.name) {
      createPlaylistMutation.mutate(formData);
    }
  };

  return (
    <Dialog.Root open={open} onOpenChange={setOpen}>
      <Dialog.Trigger asChild>
        {trigger || (
          <button className="inline-flex items-center gap-2 px-4 py-2 bg-gray-700 text-white rounded-md hover:bg-gray-600 transition-colors">
            <PlusIcon className="w-4 h-4" />
            Create Playlist
          </button>
        )}
      </Dialog.Trigger>

      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/50 z-50" />
        <Dialog.Content className="fixed top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 bg-gray-800 rounded-lg shadow-xl z-50 w-full max-w-md p-6 border border-gray-700">
          <Dialog.Title className="text-lg font-semibold mb-4 text-gray-100">
            Create New Playlist
          </Dialog.Title>

          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label htmlFor="playlist-id" className="block text-sm font-medium text-gray-300 mb-1">
                Playlist ID
              </label>
              <input
                id="playlist-id"
                type="text"
                value={formData.id}
                onChange={(e) => setFormData({ ...formData, id: e.target.value })}
                className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-gray-100 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:border-gray-500"
                placeholder="unique-playlist-id"
                required
              />
            </div>

            <div>
              <label htmlFor="playlist-name" className="block text-sm font-medium text-gray-300 mb-1">
                Display Name
              </label>
              <input
                id="playlist-name"
                type="text"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-gray-100 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:border-gray-500"
                placeholder="My Playlist"
                required
              />
            </div>

            <div>
              <label htmlFor="interval" className="block text-sm font-medium text-gray-300 mb-1">
                Interval (seconds)
              </label>
              <input
                id="interval"
                type="number"
                min="1"
                value={formData.interval_seconds}
                onChange={(e) => setFormData({ ...formData, interval_seconds: parseInt(e.target.value) || 30 })}
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
                disabled={createPlaylistMutation.isPending}
                className="px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {createPlaylistMutation.isPending ? "Creating..." : "Create"}
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