"Javascript Extension"

def prepare(pctx):
    extensions = [".ts", ".tsx"]
    q1 = configure.Query(
        extensions = extensions,
        parser = "typescript",
        expr = """
((identifier) @constant
  (#match? @constant "^[A-Z][A-Z_]+"))
        """
    )
    return configure.Metadata(
        extensions = extensions,
        queries = {
            "q1": q1
        },

    )

def declare_imports(dctx):
    pass

def declare_exports(dctx):
    pass


configure.Extension(
    name = "typescript",
    prepare = prepare,
    declare_imports = declare_imports,
    declare_exports = declare_exports,
)