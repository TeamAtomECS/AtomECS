%% Benchmark the simulation performance

thread_numbers = 1:1:12;
thread_atom_numbers = 10.^5;
steps = 1e2*5;

% Run once to force compilation.
bench(10, 1, steps);

thread_results = {};
for atom_number=thread_atom_numbers
    atom_results = {};
    for thread_number=thread_numbers
        time = bench(thread_number, atom_number, steps);
        atom_results{end+1} = struct('threads', thread_number, 'atoms', atom_number, 'time', time);
    end
    atom_results = cat(2,atom_results{:});
    thread_results{end+1} = atom_results;
end
thread_results = cat(1,thread_results{:});
save('amdahl.mat', 'thread_results', 'steps');

f = @(A, p, x) A*(1-p+p./x);
ft = fittype(f);
fitResult = fit([thread_results(end,:).threads]', [thread_results(end,:).time]', ft);
fitResult
plot([thread_results(end,:).threads]', [thread_results(end,:).time]', 'ok'); hold on;
xs = [thread_results(end,:).threads]';
xs = linspace(min(xs), max(xs), 1000);
plot(xs, fitResult(xs), 'k--'); hold off;

%%
% Plot a graph showing the results

set(gcf, 'Units', 'centimeters');
pos = get(gcf, 'Position');
set(gcf, 'Position', [ pos(1) pos(2) 9 12 ]);
clf;
axes('Units', 'centimeters', 'Position', [ 1.2 4.7 7.5 7 ]);

c1 = [ 0.1608 0.5804 0.6980 ];
c0 = 0*[ 0.0118 0.0196 0.1176 ];
get_color = @(n) interp1([0; log10(max(thread_atom_numbers))], [ c0; c1 ], log10(n));

set(gcf, 'Color', 'w');
h = [];
for i=1:size(thread_results, 1)
   h(i) = plot([thread_results(i,:).threads], 1e6*[thread_results(i,:).time]./([thread_results(i,:).atoms].*steps), '.-', 'Color', get_color(thread_results(i,1).atoms)); hold on;
   %plot([thread_results(i,:).threads], [thread_results(i,:).time], '.-',
   %'Color', get_color(thread_results(i,1).atoms)); hold on;
   %plot([thread_results(i,:).threads], [thread_results(i,1).time]./
   %[thread_results(i,:).threads], '--', 'Color',
   %get_color(thread_results(i,1).atoms)); hold on;
end
xlabel('', 'interpreter', 'latex', 'FontSize', 11);
% tau = total wall time per atom, per thread
ylabel('$\tau$ ($\mu$s) ', 'Interpreter', 'latex', 'FontSize', 11);
grid on;
set(gca, 'GridLineStyle', ':');
xlim([min(thread_numbers) max(thread_number)]);
set(get(gca, 'XAxis'), 'TickLabelInterpreter', 'Latex');
set(get(gca, 'YAxis'), 'TickLabelInterpreter', 'Latex');
set(gca, 'YScale', 'log');
set(gca, 'XTick', []);
set(gca, 'YScale', 'log');
xlim([1 6]);
labels = arrayfun(@(x) [num2str(x) ' atoms'], thread_atom_numbers, 'UniformOutput', 0);
selected = [ 1 5 length(labels) ];
legend(h(selected),labels{selected}, 'Interpreter', 'Latex');

% fit Amdahl's law and show on the graph
ax2 = axes('Units', 'centimeters', 'Position', [ 1.2 1.2 7.5 3.3 ]);
f = @(A, p, x) A*(1-p+p./x);
ft = fittype(f);
fitResult = fit([thread_results(end,:).threads]', [thread_results(end,:).time]', ft);
plot([thread_results(end,:).threads]', [thread_results(end,:).time]', 'o', 'Color', c1); hold on;
xs = [thread_results(end,:).threads]';
xs = linspace(min(xs), max(xs), 1000);
plot(xs, fitResult(xs), 'k--'); hold off;
grid on;
set(ax2, 'GridLineStyle', ':');
set(gca, 'XTick', 1:6);
set(get(ax2, 'XAxis'), 'TickLabelInterpreter', 'Latex');
set(get(ax2, 'YAxis'), 'TickLabelInterpreter', 'Latex');
xlabel('number of threads', 'interpreter', 'latex', 'FontSize', 11);
% tau = total wall time per atom, per thread
ylabel('wall time (s) ', 'Interpreter', 'latex', 'FontSize', 11);

% Render to file
%set(gcf, 'Units', 'centimeters');
%pos = get(gcf, 'Position');
%set(gcf, 'Position', [ pos(1) pos(2) 9 7.5 ]);
pos = get(gcf, 'Position');
w = pos(3); 
h = pos(4);
p = 0.01;
set(gcf,...
  'PaperUnits','centimeters',...
  'PaperPosition',[p*w p*h w h],...
  'PaperSize',[w*(1+2*p) h*(1+2*p)]);
set(gcf, 'Renderer', 'painters')
saveas(gcf, 'bench.pdf')

function loop_time = bench(thread_number, atom_numbers, steps)
   
config = struct('n_threads', int32(thread_number), 'n_steps',  int32(steps), 'n_atoms',  int32(atom_numbers));
fH = fopen('benchmark.json', 'w');
oc = onCleanup(@() fclose(fH));
fprintf(fH, '%s', jsonencode(config));
clear oc;

system('cargo run --example benchmark --release');

fH = fopen('benchmark_result.txt', 'r');
oc = onCleanup(@() fclose(fH));
simOutput = jsondecode(fgets(fH));
loop_time = simOutput.time;

end