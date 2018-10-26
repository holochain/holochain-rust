//
// Created by Nicolas Luck on 25.06.18.
//

#ifndef HOLOCHAIN_RUST_HC_CORE_C_BINDING_H
#define HOLOCHAIN_RUST_HC_CORE_C_BINDING_H

#include <stdint.h>
#include "../../dna_c_binding/include/dna_c_binding.h"
#ifdef __cplusplus
extern "C" {
#endif

typedef void Holochain;
extern Holochain *holochain_new(Dna*, const char* storage_path);
extern Holochain *holochain_load(const char* storage_path);
extern bool holochain_start(Holochain*);
extern bool holochain_stop(Holochain*);
extern char* holochain_call(Holochain*, const char* zome, const char* capability, const char* function, const char* parameters);

#ifdef __cplusplus
}
#endif


#endif //HOLOCHAIN_RUST_HC_CORE_C_BINDING_H
