const STORAGE_KEY = "missiond.admin_key";

export const adminKey = () => globalThis.localStorage?.getItem(STORAGE_KEY) ?? "";

export const setAdminKey = (key: string) => {
  if (key) {
    globalThis.localStorage?.setItem(STORAGE_KEY, key);
  }
  else {
    globalThis.localStorage?.removeItem(STORAGE_KEY);
  }
};

export const authHeaders = (): Record<string, string> => {
  const key = adminKey();

  return key ? { authorization: `Bearer ${key}` } : {};
};
