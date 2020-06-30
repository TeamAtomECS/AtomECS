%%
% Plots a graph showing the output of the zeeman_slower example.
system('cargo run --example 1d_mot --release');

output = read_output('pos.txt');
position = {output.vec};

output = read_output('vel.txt');
velocity = {output.vec};

position = cat(3, position{:});
velocity = cat(3, velocity{:});

% Plot a graph showing v_z against z.
z = squeeze(position(:,3,:));
vz = squeeze(velocity(:,3,:));

% Color code the entries by the initial velocities.
c1 = [ 0.1608 0.5804 0.6980 ];
c0 = [ 0.0118 0.0196 0.1176 ];
c = interp1([0; 120], [ c0; c1 ], vz(:,1));

clf;
for i=1:size(vz,1)
    plot(z(i,:), vz(i,:), 'k', 'Color', c(i,:)); hold on;
end
ylim([ 0 100 ]);
xlim([ -0.04 0.001 ]);
set(gcf, 'Color', 'w');
xlabel('$z$ (m)', 'interpreter', 'latex');
ylabel('$v_z$ (m/s)', 'interpreter', 'latex');
set(get(gca, 'XAxis'), 'TickLabelInterpreter', 'latex');
set(get(gca, 'YAxis'), 'TickLabelInterpreter', 'latex');
grid on;
set(gca, 'GridLineStyle', ':');

% Render to file
set(gcf, 'Units', 'centimeters');
pos = get(gcf, 'Position');
set(gcf, 'Position', [ pos(1) pos(2) 9 7.5 ]);

set(gcf, 'Units', 'centimeters');
pos = get(gcf, 'Position');
w = pos(3); 
h = pos(4);
p = 0.01;
set(gcf,...
  'PaperUnits','centimeters',...
  'PaperPosition',[p*w p*h w h],...
  'PaperSize',[w*(1+2*p) h*(1+2*p)]);
set(gcf, 'Renderer', 'painters')
saveas(gcf, '1dmot.pdf')