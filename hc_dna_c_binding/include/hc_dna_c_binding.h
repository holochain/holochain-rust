#ifndef HC_DNA_C_BINDING_H
#define HC_DNA_C_BINDING_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef void Dna;
extern Dna *hc_dna_create();
extern Dna *hc_dna_create_from_json(const char *buf);
extern void hc_dna_free(Dna *ptr);
extern char *hc_dna_to_json(const Dna *ptr);
extern void hc_dna_string_free(char *s);

extern char *hc_dna_get_name(const Dna *ptr);
extern void hc_dna_set_name(const Dna *ptr, const char *val);

extern char *hc_dna_get_description(const Dna *ptr);
extern void hc_dna_set_description(const Dna *ptr, const char *val);

extern char *hc_dna_get_version(const Dna *ptr);
extern void hc_dna_set_version(const Dna *ptr, const char *val);

extern char *hc_dna_get_uuid(const Dna *ptr);
extern void hc_dna_set_uuid(const Dna *ptr, const char *val);

extern char *hc_dna_get_dna_spec_version(const Dna *ptr);
extern void hc_dna_set_dna_spec_version(const Dna *ptr, const char *val);

#ifdef __cplusplus
}
#endif

#endif
