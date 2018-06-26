//
// Created by Nicolas Luck on 25.06.18.
//

#ifndef HOLOCHAIN_RUST_HC_CORE_C_BINDING_H
#define HOLOCHAIN_RUST_HC_CORE_C_BINDING_H

#include <stdint.h>
#include "../../hc_dna_c_binding/include/hc_dna_c_binding.h"
#ifdef __cplusplus
extern "C" {
#endif

typedef void Holochain;
extern Holochain *hc_new(Dna*);
extern bool hc_start(Holochain*);
extern bool hc_stop(Holochain*);
extern char* hc_call(Holochain*, char* zome, char* capability, char* function, char* parameters);

#ifdef __cplusplus
}
#endif


#endif //HOLOCHAIN_RUST_HC_CORE_C_BINDING_H
