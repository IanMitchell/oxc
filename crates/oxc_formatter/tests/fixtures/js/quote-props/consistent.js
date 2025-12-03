// Object where no property requires quotes - should remove all quotes in consistent mode
const noQuotesNeeded = {
  "foo": 1,
  "bar": 2,
  "baz": 3,
};

// Object where one property requires quotes - should keep all quotes in consistent mode
const quotesNeeded = {
  "foo": 1,
  "bar": 2,
  "foo-bar": 3,
};

// Object with mixed quoted and unquoted - consistent should normalize
const mixed = {
  foo: 1,
  "bar": 2,
  "hello-world": 3,
};

// Numeric keys
const numeric = {
  "1": "one",
  "2": "two",
  "3": "three",
};

// Nested objects - each should be treated independently
const nested = {
  outer: {
    "inner": 1,
    "another": 2,
  },
  "with-dash": {
    foo: 1,
    bar: 2,
  },
};

// Empty object
const empty = {};

// Single property
const single = {
  "foo": 1,
};

// Single property that requires quotes
const singleRequiresQuotes = {
  "foo-bar": 1,
};
