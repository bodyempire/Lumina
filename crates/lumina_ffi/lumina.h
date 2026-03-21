#ifndef LUMINA_H
#define LUMINA_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Opaque runtime handle */
typedef struct LuminaRuntime LuminaRuntime;

/* Create a runtime from Lumina source code.
   Returns NULL on parse/analyze error — call lumina_last_error() for details.
   Caller owns the returned handle — must call lumina_destroy() when done. */
LuminaRuntime* lumina_create(const char* source);

/* Apply a field update to a named instance.
   value_json: JSON-encoded value — "42", "true", "\"hello\""
   Returns JSON result string, or "ERROR:{...}" on rollback.
   Caller must free returned string with lumina_free_string(). */
char* lumina_apply_event(
    LuminaRuntime* runtime,
    const char* instance_name,
    const char* field_name,
    const char* value_json
);

/* Export current runtime state as JSON.
   Caller must free with lumina_free_string(). */
char* lumina_export_state(const LuminaRuntime* runtime);

/* Advance timers — call this on a regular interval (e.g. every 100ms).
   Returns JSON array of fired events.
   Caller must free with lumina_free_string(). */
char* lumina_tick(LuminaRuntime* runtime);

/* Get any strings printed by rule actions since the last call.
   Returns JSON array of strings.
   Caller must free with lumina_free_string(). */
char* lumina_get_messages(LuminaRuntime* runtime);

/* Free a string returned by any runtime function. */
void lumina_free_string(char* s);

/* Destroy the runtime and free all memory. */
void lumina_destroy(LuminaRuntime* runtime);

#ifdef __cplusplus
}
#endif

#endif /* LUMINA_H */
