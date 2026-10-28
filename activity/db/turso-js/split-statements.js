// Splits a SQL script into statements, a port of SQLite's `sqlite3_complete` state machine.

const [SEMI, WS, OTHER, EXPLAIN, CREATE, TEMP, TRIGGER, END] = [0, 1, 2, 3, 4, 5, 6, 7];
const START = 1;
// States: 0 invalid, 1 start, 2 normal, 3 explain, 4 create, 5 trigger, 6 semi, 7 end.
const TRANSITIONS = [
    // SEMI, WS, OTHER, EXPLAIN, CREATE, TEMP, TRIGGER, END
    [1, 0, 2, 3, 4, 2, 2, 2],
    [1, 1, 2, 3, 4, 2, 2, 2],
    [1, 2, 2, 2, 2, 2, 2, 2],
    [1, 3, 3, 2, 4, 2, 2, 2],
    [1, 4, 2, 2, 2, 4, 5, 2],
    [6, 5, 5, 5, 5, 5, 5, 5],
    [6, 6, 5, 5, 5, 5, 5, 7],
    [1, 7, 5, 5, 5, 5, 5, 5],
];
const KEYWORDS = new Map(Object.entries({ create: CREATE, trigger: TRIGGER, temp: TEMP, temporary: TEMP, end: END, explain: EXPLAIN }));
const ID_CHARS = /[A-Za-z0-9_$\u0080-\uffff]+/y;

// Returns the trimmed statements without their terminating semicolons, skipping empty ones.
export function splitStatements(sql) {
    const statements = [];
    let state = 0, start = 0, i = 0;
    while (i < sql.length) {
        const c = sql[i];
        let token;
        if (c === ';') {
            token = SEMI;
        } else if (' \r\t\n\f'.includes(c)) {
            token = WS;
        } else if (c === '/' && sql[i + 1] === '*') {
            const end = sql.indexOf('*/', i + 2);
            if (end < 0) throw `unterminated comment at ${i}`;
            i = end + 1;
            token = WS;
        } else if (c === '-' && sql[i + 1] === '-') {
            const end = sql.indexOf('\n', i);
            i = end < 0 ? sql.length : end;
            token = WS;
        } else if ('[`"\''.includes(c)) {
            const end = sql.indexOf(c === '[' ? ']' : c, i + 1);
            if (end < 0) throw `unterminated \`${c}\` at ${i}`;
            i = end;
            token = OTHER;
        } else {
            ID_CHARS.lastIndex = i;
            const word = ID_CHARS.exec(sql)?.[0];
            if (word) {
                i += word.length - 1;
                token = KEYWORDS.get(word.toLowerCase()) ?? OTHER;
            } else {
                token = OTHER;
            }
        }
        const previous = state;
        state = TRANSITIONS[state][token];
        if (token === SEMI && state === START) {
            if (previous > START) statements.push(sql.slice(start, i).trim());
            start = i + 1;
        }
        i += 1;
    }
    if (state > START) statements.push(sql.slice(start).trim());
    return statements;
}
