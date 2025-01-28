def what_is_bazel_version(ctx):
    blaze_version_info = ctx.use("com.google.devtools.build.lib.analysis.BlazeVersionInfo")
    print("Blaze version: {}".format(blaze_version_info.BUILD_TIMESTAMP))


urchin.patch_bazel(what_is_bazel_version)


def pray_on_action_keys(ctx):
    spawn_action = ctx.use("com.google.devtools.build.lib.analysis.actions.SpawnAction")
    spawn_action.set_implementation(
        "beforeExecute", 
        lambda this, arguments: print("SpawnAction is about to execute! action key: " % this.computeKey())
    )


urchin.patch_bazel(pray_on_action_keys)
