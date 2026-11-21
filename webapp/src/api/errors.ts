export class ApiError extends Error {
  readonly status: number;
  readonly code: string;
  constructor(status: number, code: string, message: string) {
    super(message);
    this.name = "ApiError";
    this.status = status;
    this.code = code;
  }
}

export function isApiError(e: unknown): e is ApiError {
  return e instanceof ApiError;
}

/** Plain-language text for a failed code submission (spec error vocabulary). */
export function describeCodeError(e: unknown): string {
  if (!isApiError(e)) return "Could not reach the app. Check the connection and try again.";
  switch (e.code) {
    case "UnknownVersion":
      return "This code comes from a different version. Update the addon or the app so both match.";
    case "BadCheck":
      return "The code has a typo. Copy it again from the addon window.";
    case "Truncated":
      return "The code is cut short. Copy the whole line, including the WFT1: start.";
    case "BadText":
      return "That does not look like a forever-together code. It should start with WFT1:.";
    case "TrailingBytes":
      return "The code has extra data at the end. Copy it again from the addon window.";
    case "InvalidField":
      return e.message || "A field in the code is not valid. Refresh the code in the addon and try again.";
    default:
      return e.message || "The code was refused.";
  }
}

/** True when the server refused a loopback-only route to a remote (phone) caller. */
export function isLoopbackOnly(e: unknown): boolean {
  return isApiError(e) && e.status === 403;
}
