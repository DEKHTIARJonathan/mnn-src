# Engineering rules

Keep MNN and dependencies pristine at upstream pins. Never edit either reference
repository. Keep package versions authoritative in Cargo.toml and compare the
normalized version with MNNDefine.h. Stage source archives from clean Git trees;
retain required metadata, executable modes, symlinks and notices.

Build and generate only under OUT_DIR. Never download native inputs from a Cargo
build. Run prek and formatting before packaging; enforce the strict 10,000,000
byte limit before building. Test the extracted archive with an independent
consumer and actual numerical inference. Report local and CI evidence
separately.

Preserve the small reference structure. Do not add release catalogs, generated
provenance inventories or tests of workflow configuration. Review dependency and
build-option changes on every upstream update. Never silently disable a
requested feature or patch upstream to hide failure.

Publication uses trusted publishing, exact tested archive transfer, identical
repackaging, and registry byte verification. Do not push, authenticate, publish,
or move tags without authorization. Preserve published releases.
