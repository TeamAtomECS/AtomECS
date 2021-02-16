%% Test Doppler Limit
% For the 3D MOT setup, I want to measure the velocities and verify that
% the Doppler T is obeyed.

% Get all atoms within capture region at end of simulation.

system('cargo run --example doppler_limit --release');

output_v = read_output('vel.txt');
velocities = {output_v(:).vec};
vSq = cellfun(@(v) mean(sum(v.^2,2)), velocities);
% convert to temperature
amu = 1.66e-27;
kB = 1.38e-23;
T = (amu * 87 * vSq / kB / 3);
% convert K to uK
T = T * 1e6;

clf;
plot(10*(1:length(T)), T)
fprintf('Mean T=%.2f uK\n', mean(T))
hold on
plot(xlim, [1 1 ] * 144, '--k')
set(gcf, 'Color', 'w');
xlabel('$t$ ($\mu$s)', 'interpreter', 'latex');
ylabel('T ($\mu$K)', 'interpreter', 'latex');
set(get(gca, 'XAxis'), 'TickLabelInterpreter', 'latex');
set(get(gca, 'YAxis'), 'TickLabelInterpreter', 'latex');
grid on
set(gca, 'GridLineStyle', ':');
ylim([ 0 max(ylim) ]);

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
saveas(gcf, 'doppler.pdf')