function [peakPSD, confint_psd] = getPeakPSD(position, velocity)
%%% GETPEAKPSD
%%% Fit velocity distribution in x direction to get temperature and calculate peak
%%% phase-space density over time. Input arguments are vectors from reading
%%% output of rust simulation.

%%%TODO: fit vy and vz and compare temperatures? or maybe not vz, since it
%%%might thermalise a lot faster and lead you to believe the cloud was more
%%%thermalised that it really is - maybe just average Ty and Tx?

temp = zeros(length(position),1);
ci_temp = zeros(length(position),2);
peak_n = zeros(length(position),1);

for i = 1:length(position)
    xt = position{i}(:,1);
    vxt = velocity{i}(:,1);
    
    f_vx = fitdist(vxt,'Normal');
    
    s = f_vx.sigma;
    ci_vx = paramci(f_vx);
    ci_vx = ci_vx(:,2);
    temp(i) = s^2*87*Constants.amu/Constants.kB;
    ci_temp(i,:) = ci_vx.^2*87*Constants.amu/Constants.kB;
    
    [counts, edges] = histcounts(xt);
    
    peak_n(i) = max(counts)/(mean(diff(edges)));
    
    
end


lambda_db = Constants.h./sqrt(2*pi*87*Constants.amu*Constants.kB*temp);
lambda_db_ci = Constants.h./sqrt(2*pi*87*Constants.amu*Constants.kB*ci_temp);

peakPSD = peak_n.*lambda_db.^3;
confint_psd = peak_n.*lambda_db_ci.^3;

end