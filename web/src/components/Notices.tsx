import { type FC, useSyncExternalStore } from "react";

import { dismiss, snapshot, subscribe } from "../api/notices";

export const Notices: FC = () => {
  const notices = useSyncExternalStore(subscribe, snapshot);

  if (notices.length === 0) {
    return;
  }

  return (
    <div className="fixed right-4 bottom-4 z-50 flex w-80 flex-col gap-2">
      {notices.map(notice => (
        <button
          key={notice.notice_id}
          type="button"
          onClick={() => dismiss(notice.notice_id)}
          className={[
            "cursor-pointer px-3 py-2 text-left text-sm text-white",
            notice.kind === "error" ? "bg-red-800" : "bg-gray-700",
          ].join(" ")}
        >
          {notice.message}
        </button>
      ))}
    </div>
  );
};
