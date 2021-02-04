%%
% Plots a graph showing the output of the zeeman_slower example.
system('cargo run --example molasses_1d --release');

output = read_output('vel.txt');
velocity = {output.vec};
velocity = cat(3, velocity{:});
vz = squeeze(velocity(:,3,:));
t = 10*(1:size(vz,2)); %1us timestep, output every 10

% Color code the entries by the initial velocities.
c1 = [ 0.1608 0.5804 0.6980 ];
c0 = [ 0.0118 0.0196 0.1176 ];
c = interp1([0; 30], [ c0; c1 ], vz(:,1), 'linear', 1);

clf;
for i=1:size(vz,1)
    plot(t,vz(i,:), 'k', 'Color', c(i,:)); hold on;
end
set(gcf, 'Color', 'w');
xlabel('$t$ ($\mu$s)', 'interpreter', 'latex');
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
saveas(gcf, 'molasses.pdf')

%%
% Plot scattering rates

output = read_output('scattered.txt', 'Format', '%f,%f,');
rates = {output.vec};
rates = cat(3, rates{:});
rates_a = squeeze(rates(:,1,:));
rates_b = squeeze(rates(:,2,:));
t = 10*(0:(size(rates,3)-1)); %1us timestep, output every 10

% Color code the entries by the initial velocities.
c1a = [ 0.1608 0.5804 0.6980 ];
c0a = [ 0.0118 0.0196 0.1176 ];
c1b = [ 1.0000 0.1000 0.1000 ];
c0b = [ 0.3333 0.0000 0.0000 ];
c_a = interp1([0.4; 9 ], [ c0a; c1a ], rates_a(:,1), 'linear', 1);
c_b = interp1([0.2; 1.0 ], [ c0b; c1b ], rates_b(:,1), 'linear', 1);

clf;
for i=1:size(rates,1)
    plot(t,rates_a(i,:), 'k', 'Color', c_a(i,:)); hold on;
    plot(t,rates_b(i,:), 'k', 'Color', c_b(i,:)); hold on;
end
set(gcf, 'Color', 'w');
xlabel('$t$ ($\mu$s)', 'interpreter', 'latex');
ylabel('$N_i$ ($\gamma$/step)', 'interpreter', 'latex');
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
saveas(gcf, 'rates.pdf')
