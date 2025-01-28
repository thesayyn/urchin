while (!Java.available) {}

Java.perform(() => {
    let blaze_version = Java.use("com.google.devtools.build.lib.analysis.BlazeVersionInfo");

    blaze_version.getSummary.implementation = function() {
        return "Blaze version: 3.7.0";
    }
    
    console.log(blaze_version.instance().getSummary());


    // console.log(blaze_version.instance().getReleaseName());

    // let rule_context = Java.use("com.google.devtools.build.lib.analysis.starlark.StarlarkRuleContext");

    // rule_context.$init.implementation = function(ruleContext, aspectDescriptor) {
    //     console.log("Rule context: " + ruleContext);
    //     this.$init(ruleContext, aspectDescriptor);
    // }
})

