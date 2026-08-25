import * as Dialog from "@radix-ui/react-dialog";
import { type FC, useState } from "react";

import { useTabs } from "../api/tabs";
import { useAddTabToPlaylist } from "../hooks/useAddTabToPlaylist";
import { useUpsertTab } from "../hooks/useUpsertTab";

export const AddTabDialog: FC<{ playlistId: string; }> = ({ playlistId }) => {
  const [open, setOpen] = useState(false);
  const [tabId, setTabId] = useState("");
  const [name, setName] = useState("");
  const [url, setUrl] = useState("");

  const { data: existing = [] } = useTabs();
  const addToPlaylist = useAddTabToPlaylist({ onSuccess: () => setOpen(false) });
  const upsert = useUpsertTab({
    onSuccess: created => addToPlaylist.mutate({ playlistId, tabId: created }),
  });

  const unused = existing.filter(tab => tab.tab_id !== tabId);

  const submit = () => {
    if (tabId && url) {
      upsert.mutate({ tabId, name: name || undefined, url });
    }
  };

  return (
    <Dialog.Root open={open} onOpenChange={setOpen}>
      <Dialog.Trigger className="bg-gray-800 px-2 py-1 text-sm text-gray-100 hover:bg-gray-700">
        Add tab
      </Dialog.Trigger>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/60" />
        <Dialog.Content className="fixed top-1/2 left-1/2 w-[32rem] -translate-x-1/2 -translate-y-1/2 border border-gray-800 bg-gray-950 p-5">
          <Dialog.Title className="mb-4 font-semibold text-gray-100">
            Add a tab to this playlist
          </Dialog.Title>

          {unused.length > 0 && (
            <div className="mb-4">
              <p className="mb-1 text-xs text-gray-500">Existing tabs</p>
              <div className="flex flex-wrap gap-2">
                {unused.map(tab => (
                  <button
                    key={tab.tab_id}
                    type="button"
                    onClick={() => addToPlaylist.mutate({ playlistId, tabId: tab.tab_id })}
                    className="bg-gray-800 px-2 py-1 text-sm text-gray-200 hover:bg-gray-700"
                  >
                    {tab.name}
                  </button>
                ))}
              </div>
            </div>
          )}

          <fieldset className="flex flex-col gap-3">
            <label className="text-sm text-gray-400">
              Tab id
              <input
                value={tabId}
                onChange={event => setTabId(event.target.value)}
                placeholder="grafana-overview"
                className="mt-1 w-full border border-gray-800 bg-gray-900 px-2 py-1 text-gray-100"
              />
            </label>
            <label className="text-sm text-gray-400">
              Name
              <input
                value={name}
                onChange={event => setName(event.target.value)}
                placeholder="Overview"
                className="mt-1 w-full border border-gray-800 bg-gray-900 px-2 py-1 text-gray-100"
              />
            </label>
            <label className="text-sm text-gray-400">
              URL
              <input
                value={url}
                onChange={event => setUrl(event.target.value)}
                placeholder="http://127.0.0.1:3001/d/overview?kiosk"
                className="mt-1 w-full border border-gray-800 bg-gray-900 px-2 py-1 text-gray-100"
              />
            </label>
          </fieldset>

          <div className="mt-5 flex justify-end gap-2">
            <Dialog.Close className="px-3 py-1 text-sm text-gray-400">Cancel</Dialog.Close>
            <button
              type="button"
              onClick={submit}
              disabled={!tabId || !url}
              className="bg-emerald-700 px-3 py-1 text-sm text-white disabled:opacity-50"
            >
              Add
            </button>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
};
