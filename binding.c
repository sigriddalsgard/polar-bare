#include <bare.h>
#include <js.h>

js_value_t *
polar_bare_addon_exports(js_env_t *env, js_value_t *exports);

BARE_MODULE(polar_bare_addon, polar_bare_addon_exports)
