%%
%
system('cargo run --example 2d_plus_mot_from_oven --release');
%% Load trajectories and plot results.
%
output = read_output('pos.txt');
position = {output.vec};
fprintf('Positions loaded.\n')

f = figure(1);
clf; set(gcf, 'Color', 'w');
frame = position{1};
atoms = plot3(frame(:,1), frame(:,2), frame(:,3), '.k');
view([ 45 45 ]); axis equal;
xlim([ -0.1 0.1 ]);
ylim([ -0.01 0.01 ]);
zlim([ -0.01 0.1 ]);

i=1;
while ishandle(f)
    frame = position{i};
    set(atoms, 'XData', frame(:,1), 'YData', frame(:,2), 'ZData', frame(:,3));
    i = i + 1;
    
    if i == length(position)
        i = 1;
        pause(0.2);
    end
    title(sprintf('%d', i));
    pause(0.2);
end

%%
% Identify the atoms that are pushed out of the 2D MOT, and plot their
% trajectories.

ids = [];
for frame=output'
    captured = frame.vec(:,3) > 0.015;
    ids = unique([ids; frame.id(captured)]);
end

% Note: the atoms will not have position vectors of equal length.
trajectories = cell(length(ids),1);
for i=1:length(ids)
    id = ids(i);
    for frame=output'
        mask = frame.id == id;
        trajectories{i} = [ trajectories{i}; frame.vec(mask,:) ];
    end
end

% Plot the trajectories
clf; set(gcf, 'Color', 'w');
for trajectory=trajectories'
    pos = trajectory{1};
    plot3(pos(:,1), pos(:,2), pos(:,3), '-k'); hold on;
end

axis equal;

%% 
% Close up on the source itself
xlim([ -0.01 0.01 ]);
ylim([ -0.01 0.01 ]);
zlim([ -0.01 0.01 ]);