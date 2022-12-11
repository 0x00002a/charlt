local STONKS_VALUES = {}
for n = 0, 500 do
    table.insert(STONKS_VALUES, { x = n, y = n ^ 2 })
end

return {
    type = "xy-scatter",
    datasets = {
        {
            name = "stonks",
            values = STONKS_VALUES,
        }
    },
    axis = { x = 'money (£)', y = 'time (h)' },
    steps = { x = 25, y = 100 },
    grid = { x = false, y = true }
}
