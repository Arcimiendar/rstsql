/*
declaration:
  description: test post params
  allowlist:
    body:
      - field: one
        type: integer
      - field: two
        type: object
      - field: three
        type: array
        items: 
          type: integer
  response:
    fields:
      - field: one
        type: integer
      - field: two
        type: object
      - field: three
        type: array
        items: 
          type: integer
*/
SELECT :one as one, :two as two, :three as three;