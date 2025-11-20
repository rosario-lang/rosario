#ifndef ROSARIO_BOOLEANS_DEFINED
    #define ROSARIO_BOOLEANS_DEFINED
    #if __STDC_VERSION__ < 202311l
        #define bool _Bool
        #define true 1
        #define false 0
    #endif

    #define __bool_true_false_are_defined 1
#endif
#include "stdio.h"

#define CORE_BASIC_TYPES_BOOL_FALSE false
#define CORE_BASIC_TYPES_BOOL_TRUE true

typedef struct {
    bool kind;
} core_basic_types_Bool;

core_basic_types_Bool New_core_basic_types_Bool_False() {
    core_basic_types_Bool result = { .kind = CORE_BASIC_TYPES_BOOL_FALSE,  };
    return result;
}
core_basic_types_Bool New_core_basic_types_Bool_True() {
    core_basic_types_Bool result = { .kind = CORE_BASIC_TYPES_BOOL_TRUE,  };
    return result;
}

typedef signed short int core_basic_types_Int16;

typedef signed long int core_basic_types_Int32;

typedef signed long long int core_basic_types_Int64;

typedef signed char core_basic_types_Int8;

typedef core_basic_types_Int32 core_basic_types_Integer;

typedef unsigned short int core_basic_types_UInt16;

typedef unsigned long int core_basic_types_UInt32;

typedef unsigned long long int core_basic_types_UInt64;

typedef unsigned char core_basic_types_UInt8;

typedef core_basic_types_UInt32 core_basic_types_UInteger;

#define C_MAIN_OPTION_INT32_NONE false
#define C_MAIN_OPTION_INT32_SOME true

typedef struct {
    bool kind;
    core_basic_types_Int32 Some_0;
} c_main_Option_Int32;

c_main_Option_Int32 New_c_main_Option_Int32_None() {
    c_main_Option_Int32 result = { .kind = C_MAIN_OPTION_INT32_NONE,  };
    return result;
}
c_main_Option_Int32 New_c_main_Option_Int32_Some(core_basic_types_Int32 v0) {
    c_main_Option_Int32 result = { .kind = C_MAIN_OPTION_INT32_SOME, .Some_0 = v0 };
    return result;
}

void c_main_Main() {
    const core_basic_types_Int32 a = 2;
    const core_basic_types_Int32 b = 2;
    const core_basic_types_Int32 c = a + b;
    const c_main_Option_Int32 enum_test = New_c_main_Option_Int32_Some(36);
    {
        printf("%li\n", c);
    }
}
int main() {
    c_main_Main();
    return 0;
}
