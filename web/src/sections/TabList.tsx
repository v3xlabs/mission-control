import { type FC } from "react";
import { FiTrash2 } from "react-icons/fi";

import { useTabs } from "../api/tabs";
import { useDeleteTab } from "../hooks/useDeleteTab";

/**
 * Every configured tab, whether or not a playlist uses it. A camera raised by an alert belongs to
 * no playlist, and the playlist cards are the only other place a tab is drawn.
 */
export const TabList: FC = () => {
  const { data: tabs, isLoading, error } = useTabs();
  const remove = useDeleteTab();

  if (isLoading) {
    return <p className="p-4 text-gray-500">Loading tabs...</p>;
  }

  if (error || !tabs) {
    return <p className="p-4 text-red-400">Could not load tabs.</p>;
  }

  return (
    <section className="flex flex-col gap-3 p-4">
      <h2 className="text-lg font-semibold text-gray-100">Tabs</h2>

      {tabs.length === 0
        ? <p className="text-gray-500">No tabs configured.</p>
        : (
            <ul className="border border-gray-800">
              {tabs.map(tab => (
                <li
                  key={tab.tab_id}
                  className="flex items-center gap-3 border-b border-gray-800 px-4 py-2 last:border-b-0"
                >
                  <span className="w-56 shrink-0 truncate text-gray-100">{tab.name}</span>
                  <span className="min-w-0 flex-1 truncate text-xs text-gray-500">
                    {tab.url ?? "camera"}
                  </span>
                  <button
                    type="button"
                    onClick={() => remove.mutate(tab.tab_id)}
                    title="Delete this tab"
                    className="text-gray-500 hover:text-red-400"
                  >
                    <FiTrash2 />
                  </button>
                </li>
              ))}
            </ul>
          )}
    </section>
  );
};
