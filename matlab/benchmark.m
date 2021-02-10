%% Benchmark the simulation performance

thread_numbers = 1:1:6;
thread_atom_numbers = [1e0 1e1 1e2 1e3 1e4 1e5];
atom_numbers = 10.^[0:0.5:6];
steps = 5e3;

% Run once to force compilation.
bench(10, 1, steps);

thread_results = {};
for atom_number=thread_atom_numbers
    atom_results = {};
    for thread_number=thread_numbers
        tic
        bench(thread_number, atom_number, steps);
        time = toc;
        atom_results{end+1} = struct('threads', thread_number, 'atoms', atom_number, 'time', time);
    end
    atom_results = cat(2,atom_results{:});
    thread_results{end+1} = atom_results;
end
thread_results = cat(1,thread_results{:});

%%
% Plot a graph showing the results
c1 = [ 0.1608 0.5804 0.6980 ];
c0 = [ 0.0118 0.0196 0.1176 ];
get_color = @(n) interp1([0; log10(max(thread_atom_numbers))], [ c0; c1 ], log10(n));
clf;
set(gcf, 'Color', 'w');
for i=1:size(thread_results, 1)
   plot([thread_results(i,:).threads], [thread_results(i,:).time]./([thread_results(i,:).atoms].*[thread_results(i,:).threads].*steps), '.-', 'Color', get_color(thread_results(i,1).atoms)); hold on;
   %plot([thread_results(i,:).threads], [thread_results(i,:).time], '.-',
   %'Color', get_color(thread_results(i,1).atoms)); hold on;
   %plot([thread_results(i,:).threads], [thread_results(i,1).time]./
   %[thread_results(i,:).threads], '--', 'Color',
   %get_color(thread_results(i,1).atoms)); hold on;
end
xlabel('number of threads', 'interpreter', 'latex');
% tau = total wall time per atom, per thread
ylabel('normalised step time $\tau$ (s) ', 'Interpreter', 'latex');
grid on;
set(gca, 'GridLineStyle', ':');
xlim([min(thread_numbers) max(thread_number)]);
set(get(gca, 'XAxis'), 'TickLabelInterpreter', 'Latex');
set(get(gca, 'YAxis'), 'TickLabelInterpreter', 'Latex');
set(gca, 'YScale', 'log');
set(gca, 'XTick', 1:12);
set(gca, 'YScale', 'log');
xlim([1 6]);
labels = arrayfun(@(x) [num2str(x) ' atoms'], thread_atom_numbers, 'UniformOutput', 0);
legend(labels{:});

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
saveas(gcf, 'bench.pdf')

function bench(thread_number, atom_numbers, steps)
   
config = struct('n_threads', int32(thread_number), 'n_steps',  int32(steps), 'n_atoms',  int32(atom_numbers));
fH = fopen('benchmark.json', 'w');
fprintf(fH, '%s', jsonencode(config));
fclose(fH);

system('cargo run --example benchmark --release');

end