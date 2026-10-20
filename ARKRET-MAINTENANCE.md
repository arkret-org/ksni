# Arkret zbus 6 integration

This branch migrates the D-Bus implementation to zbus 6, whose wire types are
part of zbus itself. Package versions and license terms are unchanged.

Consumer repositories must pin a complete commit revision from this branch.
The branch is an integration candidate until Linux runtime acceptance passes.

Linux integration: `dbus-run-session -- cargo test --no-default-features
--features async-io,blocking --test zbus6` exercises watcher registration, SNI
properties and activation, recursive D-Bus menu layout and click dispatch, and
shutdown using zbus 6. The pre-existing tests use the removed zbus blocking API
and still require migration before claiming the complete upstream suite.
