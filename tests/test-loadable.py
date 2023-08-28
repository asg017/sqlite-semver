import sqlite3
import unittest

EXT_PATH="./dist/debug/semver0"

def connect(ext):
  db = sqlite3.connect(":memory:")

  db.execute("create table base_functions as select name from pragma_function_list")
  db.execute("create table base_modules as select name from pragma_module_list")

  db.enable_load_extension(True)
  db.load_extension(ext)

  db.execute("create temp table loaded_functions as select name from pragma_function_list where name not in (select name from base_functions) order by name")
  db.execute("create temp table loaded_modules as select name from pragma_module_list where name not in (select name from base_modules) order by name")

  db.row_factory = sqlite3.Row
  return db


db = connect(EXT_PATH)

def explain_query_plan(sql):
  return db.execute("explain query plan " + sql).fetchone()["detail"]

def execute_all(sql, args=None):
  if args is None: args = []
  results = db.execute(sql, args).fetchall()
  return list(map(lambda x: dict(x), results))

FUNCTIONS = [
  "semver_debug",
  "semver_gt",
  "semver_matches",
  "semver_version",
  "semver_version",
  "semver_version",
  "semver_version",
  "semver_version_pointer",
  "semver_version_pointer",
  "semver_version_pointer",
]

MODULES = [
  "semver_requirements",
]
def spread_args(args):
  return ",".join(['?'] * len(args))

class TestRegex(unittest.TestCase):
  def test_funcs(self):
    funcs = list(map(lambda a: a[0], db.execute("select name from loaded_functions").fetchall()))
    self.assertEqual(funcs, FUNCTIONS)

  def test_modules(self):
    modules = list(map(lambda a: a[0], db.execute("select name from loaded_modules").fetchall()))
    self.assertEqual(modules, MODULES)

  def test_semver_version(self):
    self.assertEqual(db.execute("select semver_version()").fetchone()[0][0], "v")

  def test_semver_debug(self):
    debug = db.execute("select semver_debug()").fetchone()[0]
    self.assertEqual(len(debug.splitlines()), 3)

  def test_semver_gt(self):
    semver_gt = lambda *args: db.execute("select semver_gt()", args).fetchone()[0]
    self.skipTest("TODO")

  def test_semver_matches(self):
    semver_matches = lambda *args: db.execute("select semver_matches()", args).fetchone()[0]
    self.skipTest("TODO")

  def test_semver_version_pointer(self):
    semver_version_pointer = lambda *args: db.execute("select semver_version_pointer()", args).fetchone()[0]
    self.skipTest("TODO")

  def test_semver_requirements(self):
    semver_requirements = lambda *args: db.execute("select * from semver_requirements()", args).fetchone()[0]
    self.skipTest("TODO")

class TestCoverage(unittest.TestCase):
  def test_coverage(self):
    test_methods = [method for method in dir(TestRegex) if method.startswith('test_')]
    funcs_with_tests = set([x.replace("test_", "") for x in test_methods])

    for func in FUNCTIONS:
      self.assertTrue(func in funcs_with_tests, f"{func} does not have corresponding test in {funcs_with_tests}")

    for module in MODULES:
      self.assertTrue(module in funcs_with_tests, f"{module} does not have corresponding test in {funcs_with_tests}")

if __name__ == '__main__':
    unittest.main()
