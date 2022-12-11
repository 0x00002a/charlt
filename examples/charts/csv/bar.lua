local f, e = io.open("./examples/charts/csv/data.csv", "r")
if e ~= nil or f == nil then
    print("failed to open data file: ", e)
    return
end
local headers, data = csv.parse_file(f)
f:close()

local categories = {}
local datasets = {}

for h = 2, #headers do
    table.insert(datasets, {
        name = headers[h],
        values = {},
    })
end

for _, row in ipairs(data) do
    for h = 2, #headers do
        table.insert(datasets[h - 1].values, tonumber(row[h]))
    end
    table.insert(categories, row[1])
end
return {
    type = "bar",
    datasets = datasets,
    categories = categories,
}
