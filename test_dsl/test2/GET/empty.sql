/*
declaration:
  description: "test test2/GET/empty.sql"
  response:
    fields:
      - field: now
        description: now
        type: string
      - field: get_random_uuid
        description: rundom uuid
        type: string
      - field: json
        type: object
        fields:
          - field: a
            type: array
            items:
              type: number
      - field: bytea
        type: object
        fields:
          - field: type
            type: string
          - field: base64
            type: string 

*/
SELECT NOW(), gen_random_uuid(), '{"a": [1, 2, 3]}'::JSON, 'ASDF'::bytea;