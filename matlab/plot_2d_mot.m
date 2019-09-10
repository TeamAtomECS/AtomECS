%%
%
system('cargo run --example 2d_mot --release');

%%
% Plot the distribution of v_r as the simulation progresses.
%
% Early frames are colored a light color, later frames a darker color.

output = read_output('vel.txt');
velocity = {output.vec};

clf; set(gcf, 'Color', 'w');
c0 = [ 0.1608 0.5804 0.6980 ];
c1 = [ 0.0118 0.0196 0.1176 ];

v_r = @(v) sum(v(:,1).^2+v(:,2).^2, 2).^0.5;
clear edges
for i=1:length(velocity)
    c = interp1([0; length(velocity)], [ c0; c1 ], i);
    
    if ~exist('edges', 'var')
        [counts,edges] = histcounts(v_r(velocity{i}),300);
        centres = (edges(1:end-1)+edges(2:end))/2;
    else
        counts = histcounts(v_r(velocity{i}), edges);
    end
    plot(centres, counts, '-', 'Color', c);
    hold on;
end
hold off;

xlabel('$v_r$ (m/s)', 'Interpreter', 'Latex');
ylabel('proportion', 'Interpreter', 'Latex');
set(get(gca, 'XAxis'), 'TickLabelInterpreter', 'latex');
set(get(gca, 'YAxis'), 'TickLabelInterpreter', 'latex');
