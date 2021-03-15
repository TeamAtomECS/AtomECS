function animate(varargin)
%ANIMATE(...) Plot an animation of trajectories
%
% Input Arguments:
%  AxisView: angles for the viewport
%  AxisEqual: whether to hold x,y,z axis at same scale
%  FrameSkip: Number of frames to skip
%  HighlightFn: Function that determines which atoms to highlight.
%  SimulationRegion: Axes limits for plot.
%
% The HighlightFn is used to draw some trajectories in a different color;
% it accepts the atomic position struct as an input, and returns a vector
% of atomic IDs to be drawn differently.

ip = inputParser;
ip.addParameter('AxisView', [ 45 45 ]);
ip.addParameter('AxisEqual', 1);
ip.addParameter('FrameSkip', 1);
ip.addParameter('HighlightFn', @(position_output) [])
ip.addParameter('SimulationRegion', [ -0.1 0.1; -0.1 0.1; -0.1 0.1 ])
ip.addParameter('SaveVideo', 0)
ip.parse(varargin{:});

% Detect atoms which are not captured into the 3D MOT.
position_output = util.read_output('pos.txt');
captured_ids = ip.Results.HighlightFn(position_output);

position = {position_output.vec};
fprintf('Positions loaded.\n')

f = figure(1);
clf; set(gcf, 'Color', 'w');
frame = position{1};
atoms = scatter3(frame(:,1), frame(:,2), frame(:,3), '.k');
if ip.Results.AxisEqual
    axis equal;
end
view(ip.Results.AxisView);
sr = ip.Results.SimulationRegion;
xlim(sr(1,:));
ylim(sr(2,:));
zlim(sr(3,:));

if ip.Results.SaveVideo
    v = VideoWriter('trajetory.avi','Motion JPEG AVI');
    v.Quality = 95;
    v.FrameRate = 10;
    v.open();
    oc = onCleanup(@() v.close());
end

i=1;
while ishandle(f)
    frame = position{i};
    frame_ids = position_output(i).id;
    captured = ismember(frame_ids, captured_ids);
    color = repmat([ 0.5 0.5 0.5 ], length(captured), 1);
    color(captured, :) = repmat([ 0.8 0.2 0.2 ], sum(captured), 1);
    set(atoms, 'XData', frame(:,1), 'YData', frame(:,2), 'ZData', frame(:,3), 'CData', color);
    i = i + ip.Results.FrameSkip;
    
    if i >= length(position) || isempty(frame)
        if ip.Results.SaveVideo
            break;
        else
            i = 1;
            pause(0.2);
        end
    end
    title(sprintf('%d', i));
    if ip.Results.SaveVideo
        img = getframe(gcf);
        writeVideo(v,img);
    end
    pause(0.02);
end

clear oc;

end
