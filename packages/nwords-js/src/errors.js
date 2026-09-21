/** A codec/input error; messages never include raw input words or phrases. */
export class NwordsError extends Error {
  constructor(code, message, details = {}) {
    super(message);
    this.name = 'NwordsError';
    this.code = code;
    if (details.field !== undefined) this.field = details.field;
    if (details.position !== undefined) this.position = details.position;
  }
}

/** Asset loading is separate from ordinary codec failure. */
export class NwordsLoadError extends Error {
  constructor(code, message, options = {}) {
    super(message, options);
    this.name = 'NwordsLoadError';
    this.code = code;
  }
}
