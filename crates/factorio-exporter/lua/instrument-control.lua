export = require "export"
prototypes = require "prototypes"

script.on_nth_tick(1, function()
    export.Export("<EXPORT>")
    prototypes.export()
    export.Export("</EXPORT>")
end)
