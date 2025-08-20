/*
declaration:
  description: test get params
  allowlist:
    query:
      - field: hello
        type: integer
  response:
    fields:
      - field: ?column?
        description: test unnamed column
        type: integer

*/
SELECT 1 + :hello::INTEGER;