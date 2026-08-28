import * as Dialog from "@radix-ui/react-dialog";
import { type FC, useState } from "react";

import { useCreatePlaylist } from "../hooks/useCreatePlaylist";

export const CreatePlaylistDialog: FC = () => {
  const [open, setOpen] = useState(false);
  const [playlistId, setPlaylistId] = useState("");
  const [name, setName] = useState("");
  const [interval, setInterval] = useState("1m");

  const create = useCreatePlaylist({ onSuccess: () => setOpen(false) });

  return (
    <Dialog.Root open={open} onOpenChange={setOpen}>
      <Dialog.Trigger className="bg-gray-800 px-3 py-1 text-sm text-gray-100 hover:bg-gray-700">
        New playlist
      </Dialog.Trigger>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/60" />
        <Dialog.Content className="fixed top-1/2 left-1/2 w-[28rem] -translate-x-1/2 -translate-y-1/2 border border-gray-800 bg-gray-950 p-5">
          <Dialog.Title className="mb-4 font-semibold text-gray-100">New playlist</Dialog.Title>

          <fieldset className="flex flex-col gap-3">
            <label className="text-sm text-gray-400">
              Playlist id
              <input
                value={playlistId}
                onChange={event => setPlaylistId(event.target.value)}
                placeholder="lobby"
                className="mt-1 w-full border border-gray-800 bg-gray-900 px-2 py-1 text-gray-100"
              />
            </label>
            <label className="text-sm text-gray-400">
              Name
              <input
                value={name}
                onChange={event => setName(event.target.value)}
                placeholder="Lobby"
                className="mt-1 w-full border border-gray-800 bg-gray-900 px-2 py-1 text-gray-100"
              />
            </label>
            <label className="text-sm text-gray-400">
              Interval
              <input
                value={interval}
                onChange={event => setInterval(event.target.value)}
                placeholder="1m"
                className="mt-1 w-full border border-gray-800 bg-gray-900 px-2 py-1 text-gray-100"
              />
              <span className="mt-1 block text-xs text-gray-600">
                A duration such as 30s, 5m or 1h.
              </span>
            </label>
          </fieldset>

          <div className="mt-5 flex justify-end gap-2">
            <Dialog.Close className="px-3 py-1 text-sm text-gray-400">Cancel</Dialog.Close>
            <button
              type="button"
              onClick={() => create.mutate({
                playlist_id: playlistId,
                name: name || undefined,
                interval,
              })}
              disabled={!playlistId || !interval}
              className="bg-emerald-700 px-3 py-1 text-sm text-white disabled:opacity-50"
            >
              Create
            </button>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
};
