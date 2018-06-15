#include "test.h"
#include "../../hc_dna_c_binding/include/hc_dna_c_binding.h"

void TestHcDna::serializeAndDeserialize() {
  Dna *dna;
  Dna *dna2;
  char *buf;

  dna = hc_dna_create();
  buf = hc_dna_to_json(dna);
  hc_dna_free(dna);

  dna2 = hc_dna_create_from_json(buf);
  hc_dna_string_free(buf);

  buf = hc_dna_get_dna_spec_version(dna2);

  QCOMPARE(QString(buf), QString("2.0"));

  hc_dna_string_free(buf);
  hc_dna_free(dna2);
}

QTEST_MAIN(TestHcDna)
