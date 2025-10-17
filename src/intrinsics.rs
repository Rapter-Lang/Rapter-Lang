// Built-in intrinsic functions that map directly to C stdlib
// These don't need extern declarations - the compiler knows about them

// Simple list of intrinsic function names that don't need external declarations
const INTRINSIC_NAMES: &[&str] = &[
    // Memory allocation
    "malloc",
    "free",
    "realloc",
    "calloc",
    // String operations
    "strlen",
    "strcmp",
    "strncmp",
    "strcpy",
    "strncpy",
    "strcat",
    "strncat",
    "strchr",
    "strstr",
    "strdup",
    // Memory operations
    "memcpy",
    "memmove",
    "memset",
    "memcmp",
    // I/O operations
    "printf",
    "fprintf",
    "sprintf",
    "snprintf",
    "scanf",
    "fscanf",
    "sscanf",
    "puts",
    "fputs",
    "putchar",
    "getchar",
    "fopen",
    "fclose",
    "fread",
    "fwrite",
    "fseek",
    "ftell",
    "rewind",
    // Math operations
    "abs",
    "labs",
    "sqrt",
    "pow",
    "sin",
    "cos",
    "tan",
    "floor",
    "ceil",
    "round",
    // Conversion
    "atoi",
    "atol",
    "atof",
    "strtol",
    "strtod",
];

pub fn is_intrinsic(name: &str) -> bool {
    INTRINSIC_NAMES.contains(&name)
}
