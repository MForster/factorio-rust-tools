local export = {}

local indent = ""

function export.Export(...)
    localised_print({"", indent, ...})
end

function Indent(callback)
    local old_indent = indent
    indent = indent .. "  "
    callback()
    indent = old_indent
end

function export.ExportString(name, value)
    -- Unfortunately we have no control over the string printed by
    -- `localised_print`. There can be single/double quotes or new lines in
    -- there. Neither JSON nor YAML can deal with that well. YAML could if we
    -- had a way to control the indentation, but we don't. So, let's solve it
    -- the hacky way: post-processing from Rust.
    if value ~= nil then
        export.Export(name, ": <STRING>", value, "</STRING>")
    end
end

function export.ExportNumber(name, value)
    if value ~= nil then
        export.Export(name, ": ", value)
    end
end

function export.ExportBool(name, value)
    if value ~= nil then
        export.Export(name, ": ", value)
    end
end

function export.ExportObject(name, callback)
    export.Export(name, ":")
    Indent(callback)
end

function export.ExportArray(name, array, callback)
    export.Export(name, ":")
    if array ~= nil then
        for _, value in ipairs(array) do
            export.Export("- ")
            Indent(function()
                callback(value)
            end)
        end
    end
end

function export.ExportTable(name, table, callback)
    if table ~= nil then
        export.ExportObject(name, function()
            for key, value in pairs(table) do
                export.ExportObject(key, function()
                    callback(value)
                end)
            end
        end)
    end
end

return export
