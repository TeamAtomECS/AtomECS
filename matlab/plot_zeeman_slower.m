%%
% Plots a graph showing the output of the zeeman_slower example.

output = read_output('pos.txt');
position = {output.vec};

output = read_output('vel.txt');
velocity = {output.vec};

position = cat(3, position{:});
velocity = cat(3, velocity{:});

% Plot a graph showing v_z against z.
z = squeeze(position(:,3,:));
vz = squeeze(velocity(:,3,:));

plot(z', vz', 'k-');
ylim([ 0 120 ]);
xlim([ -0.02 0.05 ]);
set(gcf, 'Color', 'w');
xlabel('$z$ (m)', 'interpreter', 'latex');
ylabel('$v_z$ (m/s)', 'interpreter', 'latex');
set(get(gca, 'XAxis'), 'TickLabelInterpreter', 'latex');
set(get(gca, 'YAxis'), 'TickLabelInterpreter', 'latex');