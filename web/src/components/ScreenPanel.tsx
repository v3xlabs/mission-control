import { type FC, useState } from "react";
import { FiRefreshCw } from "react-icons/fi";

import { baseUrl } from "../api/api";

/**
 * The compositor's own output, which is what is genuinely on the panel. A tab preview only shows
 * what one page painted, so the two disagree whenever anything is drawn over the page.
 *
 * Capturing spawns a process, so this is fetched on demand rather than streamed, and nothing is
 * captured until it is opened.
 */
export const ScreenPanel: FC = () => {
  const [takenAt, setTakenAt] = useState<number | undefined>();
  const [hasFailed, setHasFailed] = useState(false);

  const grab = () => {
    setHasFailed(false);
    setTakenAt(Date.now());
  };

  const isOpen = takenAt !== undefined;

  // The daemon sends no-store, but a changing query keeps a re-grab from being served out of
  // the browser's in-memory image cache.
  const source = new URL(`screen?at=${takenAt}`, baseUrl);

  return (
    <section className="border border-gray-800">
      <header className="flex items-center gap-3 px-4 py-3">
        <h2 className="font-semibold text-gray-100">On screen now</h2>
        <span className="text-xs text-gray-500">
          captured from the compositor, not from a page
        </span>
        <span className="flex-1" />
        {isOpen && (
          <button
            type="button"
            onClick={grab}
            title="Capture again"
            className="text-gray-400"
          >
            <FiRefreshCw />
          </button>
        )}
        <button
          type="button"
          onClick={() => (isOpen ? setTakenAt(undefined) : grab())}
          className="bg-gray-800 px-2 py-1 text-sm text-gray-100 hover:bg-gray-700"
        >
          {isOpen ? "Hide" : "Capture"}
        </button>
      </header>

      {isOpen && (
        <div className="border-t border-gray-800 p-4">
          {hasFailed
            ? (
                <p className="text-sm text-gray-500">
                  The compositor did not answer. Check that the screenshot command in
                  display.toml is on the daemon&apos;s PATH.
                </p>
              )
            : (
                <img
                  src={source.href}
                  alt="The display's current output"
                  onError={() => setHasFailed(true)}
                  className="max-h-96 w-auto"
                />
              )}
        </div>
      )}
    </section>
  );
};
