export = require "export"

local function ExportIcon(section, name, path)
    export.Export("- section: ", section)
    export.Export("  name: ", name)
    export.Export("  path: '", path, "'")
end

export.Export("<ICONS>")

for section, elements in pairs(data.raw) do
    for _, el in pairs(elements) do
        if el.icon then
            ExportIcon(section, el.name, el.icon)
        elseif el.icons and el.icons[1] then
            ExportIcon(section, el.name, el.icons[1].icon)
        end
    end
end

export.Export("</ICONS>")
