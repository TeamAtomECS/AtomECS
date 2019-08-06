pos = read_output('output.txt');
%%
fh = gcf;
set(gcf, 'Color', 'w');
iter = 1;
while ishandle(fh) && iter <= length(pos)
    p = pos{iter};
    plot3(p(1,:), p(2,:), p(3,:), '.');
    
    iter = iter+1;
    
    axis equal;
    xlim([-0.5 0.5]);
    ylim([-0.5 0.5]);
    zlim([-0.5 0.5]);
    
    
    pause(0.01);
end

p = pos{end};
plot3(p(1,:), p(2,:), p(3,:), '.');