#include <QtTest/QtTest>

class TestHcDna: public QObject
{
  Q_OBJECT

private slots:

  void serializeAndDeserialize();
  void canGetName();
  void canSetName();
  void canGetZomeNames();
  void canGetTraitNames();
  void canGetFunctionNames();
  void canGetFunctionParameters();
};
