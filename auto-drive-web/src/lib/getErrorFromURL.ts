export type SomeError = {
  error: string;
  errorCode: string | null;
  errorDescription: string | null;
};

export const extractErrorFromURL = () => {
  let ret: SomeError | null = null;

  const params = new URLSearchParams(window.location.hash.substring(1));

  const error = params.get("error");
  const errorCode = params.get("error_code");
  const errorDescription = params.get("error_description");

  if (error) {
    ret = {
      error,
      errorCode,
      errorDescription,
    };

    // Remove the auth fragment from the URL
    history.replaceState(
      null,
      "",
      window.location.pathname + window.location.search,
    );
  }

  return ret;
};
