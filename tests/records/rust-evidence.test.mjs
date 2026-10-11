import { test } from "node:test";
import assert from "node:assert/strict";
import { validateRecords } from "../../scripts/check-product-records.mjs";
import { addFixtureFile, createFixtureTree, patchFixture } from "./fixtures.mjs";

const cap = "docs/product/capabilities/CAP-001-fixture-capability.md";

function rustReference(root, file, content) {
  addFixtureFile(root, file, content);
  patchFixture(root, cap, `\`${file}\``, "`src/engine.test.mjs`");
}

test("a CAP can cite a Rust source file containing unit tests", async () => {
  const { root, cleanup } = createFixtureTree();
  try {
    rustReference(root, "src/engine.rs", "#[cfg(test)]\nmod tests {\n    #[test]\n    fn engine_runs() { assert!(true); }\n}\n");
    assert.deepEqual((await validateRecords(root)).errors, []);
  } finally {
    cleanup();
  }
});

test("a production-only Rust file is not behaviour-test evidence", async () => {
  const { root, cleanup } = createFixtureTree();
  try {
    rustReference(root, "src/engine.rs", "pub fn engine() {}\n");
    const { errors } = await validateRecords(root);
    assert.ok(errors.some(error => error.includes("Rust test reference has no #[test] attribute")), errors.join("\n"));
  } finally {
    cleanup();
  }
});
