export type Notice = {
  notice_id: number;
  message: string;
  kind: "error" | "info";
};

// Held on one const object rather than in reassignable module bindings, so the store has a
// single owner and `useSyncExternalStore` gets a stable snapshot reference between emits.
const store = {
  notices: [] as Notice[],
  next: 1,
  listeners: new Set<() => void>(),
};

const emit = () => {
  for (const listener of store.listeners) {
    listener();
  }
};

export const subscribe = (listener: () => void) => {
  store.listeners.add(listener);

  return () => {
    store.listeners.delete(listener);
  };
};

export const snapshot = () => store.notices;

export const dismiss = (notice_id: number) => {
  store.notices = store.notices.filter(notice => notice.notice_id !== notice_id);
  emit();
};

/** Every mutation failure lands here. A refused request the reader cannot see reads as success. */
export const notify = (message: string, kind: Notice["kind"] = "error") => {
  const notice_id = store.next;

  store.next += 1;
  store.notices = [...store.notices, { notice_id, message, kind }];
  emit();

  setTimeout(() => dismiss(notice_id), 8000);
};
