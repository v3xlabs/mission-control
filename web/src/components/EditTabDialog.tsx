import { FC, useState, useEffect } from "react";
import * as Dialog from "@radix-ui/react-dialog";
import { useUpdateTab } from "../hooks/useUpdateTab";
import type { components } from "../api/schema.gen";

type TabInfo = components["schemas"]["TabInfo"];

interface EditTabDialogProps {
  tab: TabInfo | null;
  isOpen: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess?: () => void;
}

export const EditTabDialog: FC<EditTabDialogProps> = ({
  tab,
  isOpen,
  onOpenChange,
  onSuccess,
}) => {
  const [form, setForm] = useState({
    name: "",
    url: "",
    persist: true,
  });

  const updateTabMutation = useUpdateTab({
    onSuccess: () => {
      onOpenChange(false);
      onSuccess?.();
    },
  });

  useEffect(() => {
    if (tab && isOpen) {
      setForm({
        name: tab.name,
        url: tab.url,
        persist: tab.persist,
      });
    }
  }, [tab, isOpen]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!tab) return;

    updateTabMutation.mutate({
      tabId: tab.id,
      name: form.name,
      url: form.url,
      persist: form.persist,
    });
  };

  if (!tab) return null;

  return (
    <Dialog.Root open={isOpen} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/50 z-50" />
        <Dialog.Content className="fixed top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 bg-gray-800 rounded-lg shadow-xl z-50 w-full max-w-md p-6 border border-gray-700">
          <Dialog.Title className="text-lg font-semibold mb-4 text-gray-100">
            Edit Tab
          </Dialog.Title>

          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label htmlFor="tab-name" className="block text-sm font-medium text-gray-300 mb-1">
                Tab Name
              </label>
              <input
                id="tab-name"
                type="text"
                value={form.name}
                onChange={(e) => setForm({ ...form, name: e.target.value })}
                className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-gray-100 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:border-gray-500"
                placeholder="Enter tab name"
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
                value={form.url}
                onChange={(e) => setForm({ ...form, url: e.target.value })}
                className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-gray-100 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:border-gray-500"
                placeholder="https://example.com"
                required
              />
            </div>

            <div className="flex items-center">
              <input
                id="tab-persist"
                type="checkbox"
                checked={form.persist}
                onChange={(e) => setForm({ ...form, persist: e.target.checked })}
                className="rounded border-gray-600 text-green-600 focus:ring-green-500 focus:ring-offset-gray-800"
              />
              <label htmlFor="tab-persist" className="ml-2 block text-sm text-gray-300">
                Keep tab loaded in browser memory
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
                disabled={updateTabMutation.isPending}
                className="px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {updateTabMutation.isPending ? "Updating..." : "Update Tab"}
              </button>
            </div>
          </form>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
};