type AnyResponse = { status: number; data: unknown; };

export class ApiRequestError extends Error {
  readonly status: number;

  constructor(status: number, message: string) {
    super(message);
    this.name = "ApiRequestError";
    this.status = status;
  }
}

const errorMessage = (data: unknown, status: number) =>
  (typeof data === "object" && data !== null && "message" in data && typeof data.message === "string"
    ? data.message
    : `request failed with status ${status}`);

export const failure = (response: AnyResponse) =>
  new ApiRequestError(response.status, errorMessage(response.data, response.status));

export const assertRequest = (response: AnyResponse) => {
  if (response.status !== 200) {
    throw failure(response);
  }
};
